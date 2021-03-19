use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use console::{style, Term};
use dialoguer::theme::ColorfulTheme;

use lib::{
    BlockHash, PublicKey, PrivateKey,
    Address, ImplicitAddress, ImplicitOrOriginatedWithManager,
    NewTransactionOperationBuilder, NewRevealOperationBuilder, NewOperationGroup,
};
use lib::utils::parse_float_amount;
use lib::signer::{SignOperation, LocalSigner, OperationSignatureInfo};
use lib::trezor_api::{Trezor, TezosSignTx};
use lib::http_api::HttpApi;
use lib::api::*;

use crate::spinner::SpinnerBuilder;
use crate::common::{
    exit_with_error, parse_derivation_path,
    yes_no_custom_amount_input, YesNoCustomAmount,
    estimate_gas_consumption, estimate_operation_fees,
};
use crate::emojies;
use crate::trezor::trezor_execute;
use crate::commands::CommandError;

/// Create a transaction
///
/// Outputs transaction hash to stdout in case of success.
#[derive(StructOpt)]
pub struct Transfer {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    #[structopt(short = "E", long)]
    pub endpoint: String,

    #[structopt(long = "trezor")]
    pub use_trezor: bool,

    /// Address to transfer tezos from.
    ///
    /// Can either be public key hash: tz1av5nBB8Jp6VZZDBdmGifRcETaYc7UkEnU
    ///
    /// Or if --trezor flag is set, key derivation path**, like: "m/44'/1729'/0'"
    #[structopt(short, long)]
    pub from: String,

    #[structopt(short, long)]
    pub to: String,

    #[structopt(short, long)]
    pub amount: String,

    /// Specify fee for the transaction.
    ///
    /// If not specified, fee will be estimated and you will be prompted
    /// whether or not you accept estimate or would like to enter custom one.
    #[structopt(long)]
    pub fee: Option<String>,

    // NOT cli arguments

    #[structopt(skip)]
    _trezor: Option<Trezor>,

    #[structopt(skip)]
    _api: Option<HttpApi>,

    #[structopt(skip)]
    counter: Option<u64>,

    #[structopt(skip)]
    key_path: Option<Vec<u32>>,

    #[structopt(skip)]
    manager_addr: Option<ImplicitAddress>,
}

// TODO: replace with query to persistent encrypted store for keys
fn get_keys_by_addr(addr: &ImplicitAddress) -> Result<(PublicKey, PrivateKey), ()> {
    if addr != &ImplicitAddress::from_base58check("tz1av5nBB8Jp6VZZDBdmGifRcETaYc7UkEnU").unwrap() {
        return Err(());
    }
    let pub_key = "edpktywJsAeturPxoFkDEerF6bi7N41ZnQyMrmNLQ3GZx2w6nn8eCZ";
    let priv_key = "edsk37Qf3bj5actYQj38hNnu5WtbYVw3Td7dxWQnV9XhrYeBYDuSty";

    Ok((
        PublicKey::from_base58check(pub_key).unwrap(),
        PrivateKey::from_base58check(priv_key).unwrap(),
    ))
}

impl Transfer {
    fn api(&mut self) -> &mut HttpApi {
        let endpoint = self.endpoint.clone();
        self._api.get_or_insert_with(|| {
            HttpApi::new(endpoint)
        })
    }

    fn trezor(&mut self) -> &mut Trezor {
        self._trezor.get_or_insert_with(|| {
            crate::trezor::find_device_and_connect()
        })
    }

    fn get_from_addr(&mut self) -> Address {
        if self.use_trezor && self.from.starts_with("m/") {
            let key_path = self.get_key_path().unwrap();

            crate::trezor::get_address(self.trezor(), key_path).into()
        } else {
            match Address::from_base58check(&self.from) {
                Ok(addr) => addr,
                Err(_) => {
                    exit_with_error(format!(
                        "invalid {} public key hash: {}",
                        style("--from").bold(),
                        style(&self.from).red(),
                    ));
                }
            }
        }
    }

    fn get_manager_addr(&mut self) -> ImplicitAddress {
        let addr = self.get_from_addr();
        let manager = match self.manager_addr.clone() {
            Some(mgr) => { return mgr },
            None => self.api().get_contract_manager_address(&addr).unwrap(),
        };
        self.manager_addr = Some(manager.clone());
        manager
    }

    fn get_key_path(&mut self) -> Option<Vec<u32>> {
        if !self.use_trezor {
            return None;
        }

        if let Some(key_path) = self.key_path.clone() {
            return Some(key_path);
        }

        let raw_key_path = if self.from.starts_with("m/") {
            self.from.clone()
        } else {
            // TODO: add cli argument to specify key_path there.
            eprintln!(
                "{} in order to transfer using trezor, you need to manually enter the {}, from which the {} was derived.\n\n      For more about key derivation path see: {}\n",
                style("help:").yellow(),
                style("path").green(),
                style("--from").bold(),
                style("https://learnmeabitcoin.com/technical/derivation-paths").cyan(),
            );
            dialoguer::Input::with_theme(&ColorfulTheme::default())
                .with_prompt("please enter a key derivation path")
                .with_initial_text("m/44'/1729'/0'")
                .interact_text()
                .unwrap()
        };

        let key_path = parse_derivation_path(&raw_key_path).unwrap();
        self.key_path = Some(key_path.clone());

        Some(key_path)
    }

    fn get_to_addr(&self) -> Address {
        match Address::from_base58check(&self.to) {
            Ok(addr) => addr,
            Err(_) => {
                exit_with_error(format!(
                    "invalid {} public key hash: {}",
                    style("--to").bold(),
                    style(&self.to).red(),
                ));
            }
        }
    }

    fn get_amount(&self) -> u64 {
        match parse_float_amount(&self.amount) {
            Ok(amount) => amount,
            Err(_) => {
                exit_with_error(format!(
                    "invalid amount: {}",
                    style(&self.amount).bold()
                ));
            }
        }
    }

    fn get_fee(&self) -> Option<u64> {
        self.fee.as_ref().and_then(|fee| {
            match parse_float_amount(fee) {
                Ok(amount) => Some(amount),
            Err(_) => {
                exit_with_error(format!(
                    "invalid fee: {}",
                        style(fee).bold(),
                ));
            }
        }
        })
    }

    fn get_protocol_info(&mut self) -> ProtocolInfo {
        self.api().get_protocol_info().unwrap()
    }

    fn get_head_block_hash(&mut self) -> BlockHash {
        self.api().get_head_block_hash().unwrap()
    }

    fn get_counter(&mut self) -> u64 {
        let counter = self.counter.unwrap_or_else(|| {
            let addr = self.get_manager_addr();
            self.api().get_contract_counter(&addr.into()).unwrap()
        }) + 1;
        self.counter = Some(counter);
        counter
    }

    fn get_manager_public_key(&mut self, addr: &Address) -> Option<PublicKey> {
        self.api().get_manager_public_key(&addr).unwrap()
    }

    fn build_operation_group(&mut self) -> NewOperationGroup {
        let from = self.get_from_addr();
        let to = self.get_to_addr();
        let amount = self.get_amount();
        let manual_fee = self.get_fee();

        let spinner = SpinnerBuilder::new()
            .with_prefix(style("[1/4]").bold().dim())
            .with_text("fetching necessary data from the node")
            .start();

        let protocol_info = self.get_protocol_info();
        let head_block_hash = self.get_head_block_hash();

        spinner.finish();
        eprintln!(
            "{} {} {}",
            style("[1/4]").bold().green(),
            emojies::TICK,
            "fetched necessary data from the node",
        );

        let mut operation_group = NewOperationGroup::new(
            head_block_hash.clone(),
            protocol_info.next_protocol_hash,
        );

        let mut tx_op = NewTransactionOperationBuilder::new()
            .source::<ImplicitOrOriginatedWithManager>(match from.clone() {
                Address::Implicit(source) => source.into(),
                Address::Originated(addr) => {
                    let manager = match self.api().get_contract_manager_address(&addr.clone().into()) {
                        Ok(x) => x.into(),
                        // TODO: more instructive error
                        Err(_) => exit_with_error("transfering funds from contract originated after Babylon protocol change, isn't supported.")
                    };
                    addr.with_manager(manager).into()
                }
            })
            .destination(to.clone())
            .amount(amount)
            .fee(manual_fee.unwrap_or(0))
            .counter(self.get_counter())
            .gas_limit(10300)
            .storage_limit(257);

        let reveal_op = if from.is_implicit() && self.get_manager_public_key(&from).is_none() {
            let mut reveal_op = NewRevealOperationBuilder::new()
                .source(from.clone().as_implicit().unwrap())
                .fee(0)
                .counter(self.get_counter())
                .gas_limit(10300)
                .storage_limit(257);

            if self.use_trezor {
                let key_path = self.get_key_path().unwrap();
                reveal_op = reveal_op.public_key(
                    PublicKey::from_base58check(
                        &trezor_execute(
                            self.trezor().get_public_key(key_path),
                        ),
                    ).unwrap(),
                );
            } else {
                reveal_op = reveal_op.public_key(
                    get_keys_by_addr(&self.get_manager_addr()).unwrap().0,
                );
            }
            Some(reveal_op)
        } else {
            None
        };

        // estimate gas consumption and fees
        operation_group = operation_group.with_transaction(tx_op.clone().build().unwrap());
        if let Some(reveal_op) = &reveal_op {
            operation_group = operation_group.with_reveal(reveal_op.clone().build().unwrap());
        }

        let gas_consumption = estimate_gas_consumption(
            &operation_group,
            self.api(),
        ).unwrap();

        let fees = estimate_operation_fees(
            &operation_group,
            &gas_consumption,
        );

        match (gas_consumption.transaction, fees.transaction) {
            // both should always be `Some`, otherwise
            // `estimate_gas_consumption` would fail.
            (Some(estimated_gas), Some(estimated_fee)) => {
                eprintln!();

                if let Some(fee) = manual_fee.filter(|fee| *fee < estimated_fee) {
                    eprintln!(
                        "{} Entered fee({} µꜩ ) is lower than the estimated minimum fee ({} µꜩ )!\n",
                        style("[WARN]").yellow(),
                        style(fee).red(),
                        style(estimated_fee).green(),
                    );
    }
                tx_op = tx_op.gas_limit(estimated_gas);

                let input = yes_no_custom_amount_input(
                    format!(
                        "Would you like to use estimated fee({} µꜩ ),\n  or continue with specified fee({} µꜩ )",
                        style(estimated_fee).green(),
                        style(manual_fee.unwrap_or(0)).yellow(),
                    ),
                    manual_fee.map(|_| YesNoCustomAmount::No)
                        .unwrap_or(YesNoCustomAmount::Yes),
                );

                tx_op = match input {
                    YesNoCustomAmount::Custom(custom_fee) => {
                        tx_op.fee(custom_fee)
                    }
                    YesNoCustomAmount::Yes => tx_op.fee(estimated_fee),
                    YesNoCustomAmount::No => tx_op,
                };
            }
            _ => {}
        };

        match (reveal_op, gas_consumption.reveal, fees.reveal) {
            (Some(mut reveal_op), Some(estimated_gas), Some(estimated_fee)) => {
                reveal_op = reveal_op.gas_limit(estimated_gas);

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
                let is_fee_larger = tx_op
                    .get_fee()
                    .filter(|fee| *fee >= fees.total())
                    .is_some();


                if is_fee_larger {
                    eprintln!(
                        "\n       {} current fee({} µꜩ ) should be sufficient.",
                        style("HOWEVER").bold(),
                        tx_op.get_fee().unwrap(),
                    );
                }

                let input = yes_no_custom_amount_input(
                    format!(
                        "Would you like to add an estimated fee({} µꜩ ) resulting in total: {} µꜩ ",
                        style(estimated_fee).bold(),
                        style(tx_op.get_fee().unwrap_or(0) + estimated_fee).green(),
                    ),
                    manual_fee.map(|_| YesNoCustomAmount::No)
                        .unwrap_or(YesNoCustomAmount::Yes),
                );

                reveal_op = match input {
                    YesNoCustomAmount::Custom(custom_fee) => {
                        reveal_op.fee(custom_fee)
                    }
                    YesNoCustomAmount::Yes => reveal_op.fee(estimated_fee),
                    YesNoCustomAmount::No => reveal_op,
                };
                operation_group = operation_group.with_reveal(reveal_op.build().unwrap());
            },
            _ => {}
        }

        operation_group.with_transaction(tx_op.build().unwrap())
    }

    fn sign_operation(
        &mut self,
        operation_group: &NewOperationGroup,
    ) -> OperationSignatureInfo {
        let sig_info = if !self.use_trezor {
            let _spinner = SpinnerBuilder::new()
                .with_prefix(style("[2/4]").bold().dim())
                .with_text("forging the operation and signing")
                .start();
            let forged_operation = self.api().forge_operations(&operation_group).unwrap();

            let local_signer = {
                let (pub_key, priv_key) = match get_keys_by_addr(&self.get_manager_addr()) {
                    Ok(keys) => keys,
                    Err(_) => {
                        exit_with_error(format!(
                            "no local wallet with public key hash: {}",
                            style(&self.from).bold()
                        ));
                    }
                };
                LocalSigner::new(pub_key, priv_key)
            };

            local_signer.sign_operation(forged_operation.clone()).unwrap()
        } else {
            let key_path = self.get_key_path().unwrap();
            eprintln!(
                "{} -   {}",
                style("[2/4]").bold().dim(),
                "forging and signing operation using Trezor",
            );
            let mut tx: TezosSignTx = operation_group.clone().into();
            tx.set_address_n(key_path);
            let result = OperationSignatureInfo::from(
                trezor_execute(self.trezor().sign_tx(tx))
            );

            Term::stderr().clear_last_lines(1).unwrap();

            result
        };

        eprintln!(
            "{} {} {}",
            style("[2/4]").bold().green(),
            emojies::TICK,
            "operation forged and signed",
        );

        sig_info
    }

    fn confirm_operation(&mut self, operation_hash: &str) {
        let spinner = SpinnerBuilder::new()
            .with_prefix(style("[4/4]").bold().dim())
            .with_text("waiting for confirmation")
            .start();

        for _ in 0..10 {
            thread::sleep(Duration::from_secs(2));

            let status = self.api().get_pending_operation_status(&operation_hash).unwrap();
            match status {
                PendingOperationStatus::Refused => {
                    exit_with_error("operation refused");
                }
                PendingOperationStatus::Applied => {
                }
                PendingOperationStatus::Finished => {
                    break;
                }
            }
        }

        spinner.finish();
        eprintln!(
            "{} {} {}",
            style("[4/4]").bold().green(),
            emojies::TICK,
            "operation confirmed",
        );
    }

    pub fn execute(mut self) -> Result<(), CommandError> {
        let operation_group = self.build_operation_group();
        let OperationSignatureInfo {
            operation_hash,
            operation_with_signature,
            signature,
        } = self.sign_operation(&operation_group);

        let spinner = SpinnerBuilder::new()
            .with_prefix(style("[3/4]").bold().dim())
            .with_text("applying and injecting the operation")
            .start();

        self.api().preapply_operations(&operation_group, &signature).unwrap();

        self.api().inject_operations(&operation_with_signature).unwrap();

        spinner.finish();
        eprintln!(
            "{} {} {}",
            style("[3/4]").bold().green(),
            emojies::TICK,
            "applied and injected the operation",
        );

        self.confirm_operation(&operation_hash);

        eprintln!(
            "\n  {}View operation at: {}/{}",
            emojies::FINGER_POINTER_RIGHT,
            style("https://delphinet.tezblock.io/transaction").cyan(),
            style(&operation_hash).cyan(),
        );

        if !console::user_attended() {
            println!("{}", &operation_hash);
        }

        Ok(())
    }
}
