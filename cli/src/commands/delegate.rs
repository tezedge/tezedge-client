use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use console::{style, Term};
use dialoguer::theme::ColorfulTheme;

use lib::{
    BlockHash, PublicKey, PrivateKey, Address,
    ImplicitAddress, OriginatedAddressWithManager, ImplicitOrOriginatedWithManager,
    NewDelegationOperationBuilder, NewRevealOperationBuilder, NewOperationGroup,
};
use lib::utils::parse_float_amount;
use lib::signer::{SignOperation, LocalSigner, OperationSignatureInfo};
use lib::trezor_api::{Trezor, TezosSignTx};
use lib::http_api::HttpApi;
use lib::api::*;

use crate::spinner::SpinnerBuilder;
use crate::common::{exit_with_error, parse_derivation_path};
use crate::emojies;
use crate::trezor::trezor_execute;

/// Delegate balance to baker
#[derive(StructOpt)]
pub struct Delegate {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    #[structopt(short = "E", long)]
    pub endpoint: String,

    #[structopt(long = "trezor")]
    pub use_trezor: bool,

    /// Address to delegate tezos from.
    ///
    /// Can either be public key hash: tz1av5nBB8Jp6VZZDBdmGifRcETaYc7UkEnU
    ///
    /// Or if --trezor flag is set, key derivation path**, like: "m/44'/1729'/0'"
    #[structopt(short, long)]
    pub from: String,

    #[structopt(short, long)]
    pub to: String,

    #[structopt(long)]
    pub fee: String,

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

impl Delegate {
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
            None => self.api().get_manager_address(&addr).unwrap(),
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
                "{} in order to delegate using trezor, you need to manually enter the {}, from which the {} was derived.\n\n      For more about key derivation path see: {}\n",
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

        let key_path = parse_derivation_path(&raw_key_path);
        self.key_path = Some(key_path.clone());

        Some(key_path)
    }

    fn get_to_addr(&self) -> ImplicitAddress {
        match ImplicitAddress::from_base58check(&self.to) {
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

    fn get_fee(&self) -> u64 {
        match parse_float_amount(&self.fee) {
            Ok(amount) => amount,
            Err(_) => {
                exit_with_error(format!(
                    "invalid fee: {}",
                    style(&self.fee).bold()
                ));
            }
        }
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
            self.api().get_counter_for_key(&addr.into()).unwrap()
        }) + 1;
        self.counter = Some(counter);
        counter
    }

    fn get_manager_key(&mut self, addr: &Address) -> Option<String> {
        self.api().get_manager_key(&addr).unwrap()
    }

    fn build_operation_group(&mut self) -> NewOperationGroup {
        let from = self.get_from_addr();
        let to = self.get_to_addr();
        let fee = self.get_fee();

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

        let delegation_op = NewDelegationOperationBuilder::new()
            .source::<ImplicitOrOriginatedWithManager>(match from.clone() {
                Address::Implicit(source) => source.into(),
                Address::Originated(addr) => {
                    let manager = match self.api().get_manager_address(&addr.clone().into()) {
                        Ok(x) => x.into(),
                        // TODO: more instructive error
                        Err(_) => exit_with_error("delegating funds from contract originated after Babylon protocol change, isn't supported.")
                    };
                    addr.with_manager(manager).into()
                }
            })
            .delegate_to(to.clone())
            .fee(fee)
            .counter(self.get_counter())
            .gas_limit(50000)
            .storage_limit(50000)
            .build()
            .unwrap();
        operation_group = operation_group.with_operation(delegation_op);

        if from.is_implicit() && self.get_manager_key(&from).is_none() {
            let mut reveal_op = NewRevealOperationBuilder::new()
                .source(from.clone().as_implicit().unwrap())
                .fee(fee)
                .counter(self.get_counter())
                .gas_limit(50000)
                .storage_limit(50000);

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
            operation_group.with_reveal(reveal_op.build().unwrap())
        } else {
            operation_group
        }
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

    pub fn execute(mut self) {
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
    }
}
