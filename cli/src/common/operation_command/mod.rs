use std::time::Duration;
use std::thread;
use console::{style, Term};

use lib::{
    Forge, Address, ImplicitAddress, ImplicitOrOriginatedWithManager,
    NewOperationGroup, NewOperation, NewTransactionOperation, NewRevealOperation,
    NewTransactionOperationBuilder, NewDelegationOperationBuilder,
    PrivateKey, PublicKey, ToBase58Check,
};

use lib::signer::{LocalSigner, OperationSignatureInfo};
use lib::trezor_api::{Trezor, TezosSignTx};
use lib::ledger_api::Ledger;
use lib::api::*;

use cli_spinner::SpinnerBuilder;
use crate::trezor::trezor_execute;
use crate::ledger::ledger_execute;
use crate::common::{
    exit_with_error,
    yes_no_custom_amount_input, YesNoCustomAmount,
    estimate_gas_consumption, estimate_operation_fees,
};

mod raw_operation_command;
pub use raw_operation_command::*;

mod operation_command_api;
pub use operation_command_api::*;

#[derive(thiserror::Error, Debug)]
#[error("local wallet not found for a given address")]
struct WalletNotFoundError;

// TODO: replace with query to persistent encrypted store for keys
fn get_keys_by_addr(addr: &ImplicitAddress) -> Result<(PublicKey, PrivateKey), WalletNotFoundError> {
    if addr != &ImplicitAddress::from_base58check("tz1av5nBB8Jp6VZZDBdmGifRcETaYc7UkEnU").unwrap() {
        return Err(WalletNotFoundError);
    }
    let pub_key = "edpktywJsAeturPxoFkDEerF6bi7N41ZnQyMrmNLQ3GZx2w6nn8eCZ";
    let priv_key = "edsk37Qf3bj5actYQj38hNnu5WtbYVw3Td7dxWQnV9XhrYeBYDuSty";

    Ok((
        PublicKey::from_base58check(pub_key).unwrap(),
        PrivateKey::from_base58check(priv_key).unwrap(),
    ))
}

pub struct OperationOptions {
}

pub struct OperationCommandState {
    pub version: Option<VersionInfo>,
    pub counter: Option<u64>,
    pub manager_address: Option<ImplicitAddress>,
}

impl Default for OperationCommandState {
    fn default() -> Self {
        Self {
            version: None,
            counter: None,
            manager_address: None,
        }
    }
}

pub struct TrezorState {
    trezor: Trezor,
    key_path: Vec<u32>,
}

pub struct LedgerState {
    ledger: Ledger,
    key_path: Vec<u32>,
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum OperationType {
    Transaction { amount: u64 },
    Delegation,
}

pub struct OperationCommand {
    pub options: OperationOptions,
    pub from: Address,
    pub to: Address,
    pub fee: Option<u64>,

    pub api: Box<dyn OperationCommandApi>,
    pub state: OperationCommandState,
    /// If `Some`, Trezor will be used to execute an operation.
    pub trezor_state: Option<TrezorState>,
    /// If `Some`, Ledger will be used to execute an operation.
    pub ledger_state: Option<LedgerState>,
}

impl OperationCommand {
    fn get_version(&mut self) -> GetVersionInfoResult {
        let version = self.state.version.as_ref()
            .map(|version| Ok(version.clone()))
            .unwrap_or_else(|| {
                self.api.get_version_info()
            })?;

        self.state.version.replace(version.clone());
        Ok(version)
    }

    fn get_counter(&mut self) -> Result<u64, GetContractCounterError> {
        let counter = self.state.counter
            .map(|value| Ok(value))
            .unwrap_or_else(|| {
                self.api.get_contract_counter(&self.from)
            })? + 1;

        self.state.counter.replace(counter);
        Ok(counter)
    }

    fn get_manager_address(&mut self) -> GetContractManagerAddressResult {
        let manager = self.state.manager_address.as_ref()
            .map(|addr| Ok(addr.clone()))
            .unwrap_or_else(|| {
                self.api.get_contract_manager_address(&self.from)
            })?;

        self.state.manager_address.replace(manager.clone());
        Ok(manager)
    }

    fn get_manager_public_key(&mut self) -> GetManagerPublicKeyResult {
        self.api.get_manager_public_key(&self.from)
    }

    fn build_reveal(&mut self) -> Result<Option<NewRevealOperation>, Error> {
        let source = match &self.from {
            Address::Implicit(addr) => addr.clone(),
            // If address is for originated contract, that means manager of
            // the contract created origination operation, for which he
            // would have to reveal public key as well. So no need to check
            // further if manager's public key is revealed.
            Address::Originated(_) => { return Ok(None) },
        };

        if self.get_manager_public_key()?.is_some() {
            return Ok(None);
        }

        let public_key = if let Some(trezor_state) = self.trezor_state.as_mut() {
            PublicKey::from_base58check(
                &trezor_execute(
                    trezor_state.trezor.get_public_key(trezor_state.key_path.clone())
                ),
            )?
        } else if let Some(ledger_state) = self.ledger_state.as_mut() {
            ledger_execute(
                ledger_state.ledger.get_public_key(ledger_state.key_path.clone(), false)
            )
        } else {
            match get_keys_by_addr(&self.get_manager_address()?) {
                Ok(keys) => keys.0,
                Err(_) => {
                    exit_with_error(format!(
                        "no local wallet found with address: {}",
                        style(&self.from.to_base58check()).bold(),
                    ));
                }
            }
        };
        Ok(Some(NewRevealOperation {
            source,
            public_key,
            counter: self.get_counter()?,
            fee: 0,
            gas_limit: 10300,
            storage_limit: 257,
        }))
    }

    fn build_transaction(
        &mut self,
        source: ImplicitOrOriginatedWithManager,
        amount: u64,
    ) -> Result<NewTransactionOperation, Error>
    {
        Ok(NewTransactionOperationBuilder {
            amount,
            source,
            destination: self.to.clone(),
            counter: self.get_counter()?,
            fee: self.fee.clone().unwrap_or(0),
            gas_limit: 10300,
            storage_limit: 257,
        }.build())
    }

    fn build_delegation(
        &mut self,
        source: ImplicitOrOriginatedWithManager,
    ) -> Result<NewOperation, Error>
    {
        let delegate_to = match &self.to {
            Address::Implicit(addr) => addr.clone(),
            Address::Originated(_) => {
                exit_with_error("delegating to originated account is not supported!");
            }
        };

        Ok(NewDelegationOperationBuilder {
            source,
            delegate_to: Some(delegate_to),
            counter: self.get_counter()?,
            fee: self.fee.clone().unwrap_or(0),
            gas_limit: 10300,
            storage_limit: 257,
        }.build())
    }

    fn build_operation_group(
        &mut self,
        op_type: OperationType,
    ) -> Result<NewOperationGroup, Error>
    {
        let source = match self.from.clone() {
            Address::Implicit(source) => source.into(),
            Address::Originated(addr) => {
                let manager = match self.api.get_contract_manager_address(&addr.clone().into()) {
                    Ok(x) => x.into(),
                    // TODO: more instructive error
                    Err(_) => exit_with_error("transfering funds from contract originated after Babylon protocol change, isn't supported.")
                };
                addr.with_manager(manager).into()
            }
        };

        let spinner = SpinnerBuilder::new()
            .with_prefix(style("[1/4]").bold().dim())
            .with_text("fetching necessary data from the node")
            .start();

        self.get_version()?;
        let protocol_info = self.api.get_protocol_info()?;
        let head_block_hash = self.api.get_head_block_hash()?;

        let mut operation_group = NewOperationGroup::new(
            head_block_hash.clone(),
            protocol_info.next_protocol_hash,
        );

        spinner.finish_succeed("fetched necessary data from the node");

        if let Some(reveal_op) = self.build_reveal()? {
            operation_group = operation_group.with_reveal(reveal_op);
        }


        Ok(operation_group.with_operation(match op_type {
            OperationType::Transaction { amount } => {
                self.build_transaction(source, amount)?.into()
            }
            OperationType::Delegation => {
                self.build_delegation(source)?
            }
        }))
    }

    fn estimate_and_set_fees(
        &mut self,
        operation_group: &mut NewOperationGroup,
    ) -> Result<(), Error> {
        let manual_fee = self.fee.clone();
        let gas_consumption = estimate_gas_consumption(
            &operation_group,
            &mut *self.api,
        )?;

        let fees = estimate_operation_fees(
            &operation_group,
            &gas_consumption,
        );

        let tx_gas_fee = gas_consumption.transaction
            .and_then(|gas| fees.transaction.map(|fee| (gas, fee)));

        let del_gas_fee = gas_consumption.delegation
            .and_then(|gas| fees.delegation.map(|fee| (gas, fee)));

        let op_fee = match (tx_gas_fee.clone(), del_gas_fee.clone()) {
            // both should always be `Some`, otherwise
            // `estimate_gas_consumption` would fail.
            (Some((estimated_gas, estimated_fee)), None)
                | (None, Some((estimated_gas, estimated_fee)))
            => {
                eprintln!();

                if let Some(fee) = manual_fee.filter(|fee| *fee < estimated_fee) {
                    eprintln!(
                        "{} Entered fee({} µꜩ ) is lower than the estimated minimum fee ({} µꜩ )!\n",
                        style("[WARN]").yellow(),
                        style(fee).red(),
                        style(estimated_fee).green(),
                    );
                }

                let input = yes_no_custom_amount_input(
                    format!(
                        "Would you like to use estimated fee({} µꜩ ),\n  or continue with specified fee({} µꜩ )\n",
                        style(estimated_fee).green(),
                        style(manual_fee.unwrap_or(0)).yellow(),
                    ),
                    manual_fee.map(|_| YesNoCustomAmount::No)
                        .unwrap_or(YesNoCustomAmount::Yes),
                );

                let fee = match input {
                    YesNoCustomAmount::Custom(custom_fee) => custom_fee,
                    YesNoCustomAmount::Yes => estimated_fee,
                    YesNoCustomAmount::No => manual_fee.unwrap_or(0),
                };

                if let Some(tx_op) = tx_gas_fee.and(operation_group.transaction.as_mut()) {
                    tx_op.gas_limit = estimated_gas;
                    tx_op.fee = fee;
                } else if let Some(del_op) = del_gas_fee.and(operation_group.delegation.as_mut()) {
                    del_op.gas_limit = estimated_gas;
                    del_op.fee = fee;
                }

                fee
            }
            // TODO: this should be impossible. Show error if it happens.
            _ => 0,
        };

        match (operation_group.reveal.as_mut(), gas_consumption.reveal, fees.reveal) {
            (Some(reveal_op), Some(estimated_gas), Some(estimated_fee)) => {
                reveal_op.gas_limit = estimated_gas;

                eprintln!(
                    "\n{} Account from which you are sending from, hasn't yet been {}!",
                    style("[WARN]").yellow(),
                    style("revealed").bold(),
                );
                eprintln!(
                    "\n       Additional fee (estimated {} µꜩ ) is required to reveal the account.",
                    style(estimated_fee).green(),
                );

                // whether or not entered fee is greater or equal to the total estimated fee.
                let is_fee_larger =
                    operation_group.transaction.as_ref().map(|op| op.fee)
                    .or(operation_group.delegation.as_ref().map(|op| op.fee))
                    .filter(|fee| *fee >= fees.total())
                    .is_some();


                if is_fee_larger {
                    eprintln!(
                        "\n       {} current fee({} µꜩ ) should be sufficient.",
                        style("HOWEVER").bold(),
                        manual_fee.unwrap_or(0),
                    );
                }

                let input = yes_no_custom_amount_input(
                    format!(
                        "Would you like to add an estimated fee({} µꜩ ) resulting in total: {} µꜩ ",
                        style(estimated_fee).bold(),
                        style(estimated_fee + op_fee).green(),
                    ),
                    manual_fee.map(|_| YesNoCustomAmount::No)
                        .unwrap_or(YesNoCustomAmount::Yes),
                );

                match input {
                    YesNoCustomAmount::Custom(custom_fee) => {
                        reveal_op.fee = custom_fee
                    }
                    YesNoCustomAmount::Yes => reveal_op.fee = estimated_fee,
                    YesNoCustomAmount::No => {}
                };
            },
            _ => {}
        }

        Ok(())
    }

    fn sign_operation(
        &mut self,
        operation_group: &NewOperationGroup,
    ) -> Result<OperationSignatureInfo, Error>
    {
        if let Some(trezor_state) = self.trezor_state.as_mut() {
            eprintln!(
                "{} -   {}",
                style("[2/4]").bold().dim(),
                "forging and signing operation using Trezor",
            );
            let mut tx: TezosSignTx = operation_group.clone().into();
            tx.set_address_n(trezor_state.key_path.clone());
            let sig_info = OperationSignatureInfo::from(
                trezor_execute(trezor_state.trezor.sign_tx(tx))
            );

            Term::stderr().clear_last_lines(1)?;
            eprintln!(
                "{} {} {}",
                style("[2/4]").bold().green(),
                emojies::TICK,
                "operation forged and signed",
            );

            Ok(sig_info)
        } else if let Some(ledger_state) = self.ledger_state.as_mut() {
            eprintln!(
                "{} -   {}\n",
                style("[2/4]").bold().dim(),
                "signing operation using Ledger. Please confirm an operation on your ledger.",
            );

            eprintln!("please confirm an operation on Ledger once you see the dialog on the device.\n");

            let sig_info = ledger_execute(
                ledger_state.ledger.sign_tx(
                    ledger_state.key_path.clone(),
                    operation_group.forge(),
                )
            );

            Term::stderr().clear_last_lines(4)?;
            eprintln!(
                "{} {} {}",
                style("[2/4]").bold().green(),
                emojies::TICK,
                "operation signed",
            );

            Ok(sig_info)
        } else {
            let spinner = SpinnerBuilder::new()
                .with_prefix(style("[2/4]").bold().dim())
                .with_text("forging the operation and signing")
                .start();
            let forged_operation = operation_group.forge();

            let local_signer = {
                let (pub_key, priv_key) = match get_keys_by_addr(&self.get_manager_address()?) {
                    Ok(keys) => keys,
                    Err(err) => {
                        spinner.finish_fail(format!(
                            "no local wallet found with address: {}",
                            style(&self.from.to_base58check()).bold()
                        ));
                        return Err(err.into());
                    }
                };
                LocalSigner::new(pub_key, priv_key)
            };

            let sig_info = local_signer.sign_forged_operation_bytes(
                forged_operation.as_ref(),
            );

            spinner.finish_succeed("operation forged and signed");
            Ok(sig_info)
        }
    }

    fn confirm_operation(&mut self, operation_hash: &str) -> Result<(), Error> {
        let spinner = SpinnerBuilder::new()
            .with_prefix(style("[4/4]").bold().dim())
            .with_text("waiting for confirmation")
            .start();

        for _ in 0..10 {
            thread::sleep(Duration::from_secs(2));

            let status = self.api.get_pending_operation_status(&operation_hash)?;
            match status {
                PendingOperationStatus::Refused => {
                    spinner.finish_fail("operation_refused");
                    return Ok(());
                }
                PendingOperationStatus::Applied => {
                }
                PendingOperationStatus::Finished => {
                    break;
                }
            }
        }

        spinner.finish_succeed("operation confirmed");

        Ok(())
    }

    fn execute(&mut self, op_type: OperationType) -> Result<(), Error> {
        let mut operation_group = self.build_operation_group(op_type)?;
        self.estimate_and_set_fees(&mut operation_group)?;
        let OperationSignatureInfo {
            operation_hash,
            operation_with_signature,
            signature,
        } = self.sign_operation(&operation_group)?;

        let spinner = SpinnerBuilder::new()
            .with_prefix(style("[3/4]").bold().dim())
            .with_text("applying and injecting the operation")
            .start();

        self.api.preapply_operations(&operation_group, &signature)?;

        self.api.inject_operations(&operation_with_signature)?;

        spinner.finish_succeed("applied and injected the operation");

        self.confirm_operation(&operation_hash)?;

        let version = self.get_version()?;
        if let Some(explorer_url) = version.explorer_url() {
            eprintln!(
                "\n  {}View operation at: {}/{}",
                emojies::FINGER_POINTER_RIGHT,
                style(explorer_url).cyan(),
                style(&operation_hash).cyan(),
            );
        } else {
            eprintln!(
                "\n{} couldn't find explorer url for current network({}).",
                style("[WARN]").yellow(),
                style(&version.network_version.chain_name).bold(),
            );

            eprintln!("\nOperation hash: {}", style(&operation_hash).green());
        }
        

        if !console::user_attended() {
            println!("{}", &operation_hash);
        }

        Ok(())
    }

    pub fn transfer(&mut self, amount: u64) -> Result<(), Error> {
        let op_type = OperationType::Transaction { amount };
        self.execute(op_type)
    }

    pub fn delegate(&mut self) -> Result<(), Error> {
        let op_type = OperationType::Delegation;
        self.execute(op_type)
    }
}

type Error = Box<dyn std::error::Error>;
