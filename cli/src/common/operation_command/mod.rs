use std::time::Duration;
use std::thread;
use console::{style, Term};

use lib::{
    Forge, Address, ImplicitAddress, ImplicitOrOriginatedWithManager,
    NewOperationGroup, NewOperation, NewTransactionOperation, NewRevealOperation,
    NewTransactionOperationBuilder, NewDelegationOperationBuilder,
    KeyDerivationPath, PrivateKey, PublicKey,
};

use lib::signer::{LocalSigner, OperationSignatureInfo};
use lib::explorer_api::TzStats;
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

/// Exit and print error that no wallet type(trezor, ledger, local) was selected.
fn exit_with_error_no_wallet_type_selected() -> ! {
    exit_with_error(format!(
        "{}\n{}",
        "trezor, ledger, or local wallet needs to be used to create this operation. Neither selected.",
        "Technically this shouldn't be possible!",
    ))
}

pub struct OperationOptions {
    pub no_prompt: bool,
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
    pub trezor: Trezor,
    pub key_path: KeyDerivationPath,
}

pub struct LedgerState {
    pub ledger: Ledger,
    pub key_path: KeyDerivationPath,
}

pub struct LocalWalletState {
    pub public_key: PublicKey,
    pub private_key: PrivateKey,
}

impl LocalWalletState {
    pub fn signer(&self) -> LocalSigner {
        LocalSigner::new(
            self.public_key.clone(),
            self.private_key.clone(),
        )
    }
}

#[derive(PartialEq, Debug, Clone)]
enum OperationType {
    Transaction { to: Address, amount: u64 },
    Delegation { to: Option<ImplicitAddress> },
}

pub struct OperationCommand {
    pub options: OperationOptions,
    pub from: Address,
    pub fee: Option<u64>,

    pub api: Box<dyn OperationCommandApi>,
    pub state: OperationCommandState,
    /// If `Some`, Trezor will be used to execute an operation.
    pub trezor_state: Option<TrezorState>,
    /// If `Some`, Ledger will be used to execute an operation.
    pub ledger_state: Option<LedgerState>,
    /// If `Some`, Local wallet will be used to execute an operation.
    pub local_state: Option<LocalWalletState>,
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

    fn get_counter(&mut self) -> Result<u64, Error> {
        let counter = if let Some(counter) = self.state.counter {
            counter
        } else {
            let address = self.get_manager_address()?;
            self.api.get_contract_counter(&address)?
        } + 1;

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
                    trezor_state.trezor.get_public_key(&trezor_state.key_path)
                ),
            )?
        } else if let Some(ledger_state) = self.ledger_state.as_mut() {
            ledger_execute(
                ledger_state.ledger.get_public_key(&ledger_state.key_path, false)
            )
        } else if let Some(local_state) = self.local_state.as_ref() {
            local_state.public_key.clone()
        } else {
            exit_with_error_no_wallet_type_selected()
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
        destination: Address,
        amount: u64,
    ) -> Result<NewTransactionOperation, Error>
    {
        Ok(NewTransactionOperationBuilder {
            amount,
            source,
            destination,
            counter: self.get_counter()?,
            fee: self.fee.clone().unwrap_or(0),
            gas_limit: 10300,
            storage_limit: 257,
        }.build())
    }

    fn build_delegation(
        &mut self,
        source: ImplicitOrOriginatedWithManager,
        destination: Option<ImplicitAddress>,
    ) -> Result<NewOperation, Error>
    {
        Ok(NewDelegationOperationBuilder {
            source,
            delegate_to: destination,
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
            OperationType::Transaction { to, amount } => {
                self.build_transaction(source, to, amount)?.into()
            }
            OperationType::Delegation { to } => {
                self.build_delegation(source, to)?
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

                let default_input = manual_fee.map(|_| YesNoCustomAmount::No)
                        .unwrap_or(YesNoCustomAmount::Yes);
                let input = if self.options.no_prompt {
                    default_input
                } else {
                    yes_no_custom_amount_input(
                        format!(
                            "Would you like to use estimated fee({} µꜩ ),\n  or continue with specified fee({} µꜩ )\n",
                            style(estimated_fee).green(),
                            style(manual_fee.unwrap_or(0)).yellow(),
                        ),
                        default_input,
                    )
                };

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

                let default_input = manual_fee.map(|_| YesNoCustomAmount::No)
                    .unwrap_or(YesNoCustomAmount::Yes);

                let input = if self.options.no_prompt {
                    default_input
                } else {
                    yes_no_custom_amount_input(
                        format!(
                            "Would you like to add an estimated fee({} µꜩ ) resulting in total: {} µꜩ ",
                            style(estimated_fee).bold(),
                            style(estimated_fee + op_fee).green(),
                        ),
                        default_input,
                    )
                };

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
            tx.set_address_n(trezor_state.key_path.clone().take());
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
                    &ledger_state.key_path,
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
        } else if let Some(state) = self.local_state.as_ref() {
            let spinner = SpinnerBuilder::new()
                .with_prefix(style("[2/4]").bold().dim())
                .with_text("forging the operation and signing")
                .start();
            let forged_operation = operation_group.forge();

            let sig_info = state.signer().sign_forged_operation_bytes(
                forged_operation.as_ref(),
            );

            spinner.finish_succeed("operation forged and signed");
            Ok(sig_info)
        } else {
            exit_with_error_no_wallet_type_selected()
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
        let network = version.get_network();

        match TzStats::new(network) {
            Ok(tzstats) => {
                eprintln!(
                    "\n  {}View operation at: {}/{}",
                    emojies::FINGER_POINTER_RIGHT,
                    style(tzstats.operation_link_prefix()).cyan(),
                    style(&operation_hash).cyan(),
                );
            }
            Err(err) => {
                eprintln!(
                    "\n{} {}",
                    style("[WARN]").yellow(),
                    err,
                );
                eprintln!("\nOperation hash: {}", style(&operation_hash).green());
            }
        };

        if !console::user_attended() {
            println!("{}", &operation_hash);
        }

        Ok(())
    }

    pub fn transfer(&mut self, to: Address, amount: u64) -> Result<(), Error> {
        let op_type = OperationType::Transaction { to, amount };
        self.execute(op_type)
    }

    pub fn delegate(&mut self, to: Option<ImplicitAddress>) -> Result<(), Error> {
        let op_type = OperationType::Delegation { to };
        self.execute(op_type)
    }
}

type Error = Box<dyn std::error::Error>;
