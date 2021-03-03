use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use console::{style, Term};

use lib::{
    BlockHash, PublicKeyHash, PublicKey, PrivateKey,
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
}

// TODO: replace with query to persistent encrypted store for keys
fn get_keys_by_pkh(pkh: &String) -> Result<(PublicKey, PrivateKey), ()> {
    if pkh != "tz1av5nBB8Jp6VZZDBdmGifRcETaYc7UkEnU" {
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

    fn get_from_pkh(&mut self) -> PublicKeyHash {
        if self.use_trezor {
            let raw_key_path = &self.from;

            if !raw_key_path.starts_with("m/") {
                exit_with_error(format!(
                    "when using Trezor, {} needs to be key derivation path like: \"{}\", but got: \"{}\"",
                    style("--from").bold(),
                    style("m/44'/1729'/0'").green(),
                    style(raw_key_path).red(),
                ));
            }

            let key_path = parse_derivation_path(raw_key_path);
            self.key_path = Some(key_path.clone());
            crate::trezor::get_pkh(self.trezor(), key_path)
        } else {
            match PublicKeyHash::from_base58check(&self.from) {
                Ok(pkh) => pkh,
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

    fn get_to_pkh(&self) -> PublicKeyHash {
        match PublicKeyHash::from_base58check(&self.to) {
            Ok(pkh) => pkh,
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

    fn get_counter(&mut self, pkh: &PublicKeyHash) -> u64 {
        let counter = self.counter.unwrap_or_else(|| {
            self.api().get_counter_for_key(&pkh).unwrap()
        }) + 1;
        self.counter = Some(counter);
        counter
    }

    fn get_manager_key(&mut self, pkh: &PublicKeyHash) -> Option<String> {
        self.api().get_manager_key(&pkh).unwrap()
    }

    fn build_operation_group(&mut self) -> NewOperationGroup {
        let from = self.get_from_pkh();
        let to = self.get_to_pkh();
        let fee = self.get_fee();

        let spinner = SpinnerBuilder::new()
            .with_prefix(style("[1/4]").bold().dim())
            .with_text("fetching necessary data from the node")
            .start();

        let protocol_info = self.get_protocol_info();
        let head_block_hash = self.get_head_block_hash();
        let manager_key = self.get_manager_key(&from);

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
            .source(from.clone())
            .delegate_to(to.clone())
            .fee(fee)
            .counter(self.get_counter(&from))
            .gas_limit(50000)
            .storage_limit(50000)
            .build()
            .unwrap();
        operation_group = operation_group.with_delegation(delegation_op);

        if manager_key.is_none() {
            let mut reveal_op = NewRevealOperationBuilder::new()
                .source(from.clone())
                .fee(fee)
                .counter(self.get_counter(&from))
                .gas_limit(50000)
                .storage_limit(50000);

            if self.use_trezor {
                let key_path = self.key_path.clone().unwrap();
                reveal_op = reveal_op.public_key(
                    PublicKey::from_base58check(
                        &trezor_execute(
                            self.trezor().get_public_key(key_path),
                        ),
                    ).unwrap(),
                );
            } else {
                reveal_op = reveal_op.public_key(
                    get_keys_by_pkh(&self.from).unwrap().0,
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
                let (pub_key, priv_key) = match get_keys_by_pkh(&self.from) {
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
            eprintln!(
                "{} -   {}",
                style("[2/4]").bold().dim(),
                "forging and signing operation using Trezor",
            );
            let mut tx: TezosSignTx = operation_group.clone().into();
            tx.set_address_n(self.key_path.clone().unwrap());
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
