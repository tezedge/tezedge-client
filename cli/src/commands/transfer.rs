use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use console::{style, Term};

use lib::{NewRevealOperationBuilder, ToBase58Check, api::*, signer::OperationSignatureInfo, trezor_api::TezosSignTx};
use lib::{PublicKeyHash, PublicKey, PrivateKey, NewOperationGroup, NewTransactionOperationBuilder};
use lib::utils::parse_float_amount;
use lib::signer::{SignOperation, LocalSigner};

use crate::spinner::SpinnerBuilder;
use crate::common::{exit_with_error, parse_derivation_path};
use crate::emojies;
use crate::trezor::trezor_execute;

/// Create a transaction
///
/// Outputs transaction hash to stdout in case of success.
#[derive(StructOpt, Debug, Clone)]
pub struct Transfer {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    #[structopt(short = "E", long)]
    endpoint: String,

    #[structopt(long = "trezor")]
    use_trezor: bool,

    /// Address to transfer tezos from.
    ///
    /// Can either be public key hash: tz1av5nBB8Jp6VZZDBdmGifRcETaYc7UkEnU
    ///
    /// Or if --trezor flag is set, key derivation path**, like: "m/44'/1729'/0'"
    #[structopt(short, long)]
    from: String,

    #[structopt(short, long)]
    to: String,

    #[structopt(short, long)]
    amount: String,

    #[structopt(long)]
    fee: String,
}

// TODO: replace with query to persistent encrypted store for keys
fn get_keys_by_pkh(pkh: &PublicKeyHash) -> Result<(PublicKey, PrivateKey), ()> {
    if pkh != &PublicKeyHash::from_base58check("tz1av5nBB8Jp6VZZDBdmGifRcETaYc7UkEnU").unwrap() {
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
    // TODO: fix transfer not working to new account
    pub fn execute(self) {
        let Transfer {
            // TODO: use verbose to print additional info
            verbose: _,
            endpoint,
            use_trezor,
            to,
            from: raw_from,
            amount: raw_amount,
            fee: raw_fee,
        } = self;

        let mut trezor = None;
        // key derivation path when using trezor
        let mut key_path = None;

        let from = if use_trezor {
            if !raw_from.starts_with("m/") {
                exit_with_error(format!(
                    "when using Trezor, {} needs to be key derivation path like: \"{}\", but got: \"{}\"",
                    style("--from").bold(),
                    style("m/44'/1729'/0'").green(),
                    style(raw_from).red(),
                ));
            }
            trezor = Some(crate::trezor::find_device_and_connect());
            let path = parse_derivation_path(&raw_from);
            key_path = Some(path.clone());
            crate::trezor::get_pkh(trezor.as_mut().unwrap(), path)
        } else {
            match PublicKeyHash::from_base58check(&raw_from) {
                Ok(pkh) => pkh,
                Err(_) => {
                    exit_with_error(format!(
                        "invalid {} public key hash: {}",
                        style("--from").bold(),
                        style(raw_from).magenta(),
                    ));
                }
            }
        };

        let to = match PublicKeyHash::from_base58check(&to) {
            Ok(pkh) => pkh,
            Err(_) => {
                exit_with_error(format!(
                    "invalid {} public key hash: {}",
                    style("--to").bold(),
                    style(to).magenta(),
                ));
            }
        };

        let amount = match parse_float_amount(&raw_amount) {
            Ok(amount) => amount,
            Err(_) => {
                exit_with_error(format!(
                    "invalid amount: {}",
                    style(&raw_amount).bold()
                ));
            }
        };

        let fee = match parse_float_amount(&raw_fee) {
            Ok(amount) => amount,
            Err(_) => {
                exit_with_error(format!(
                    "invalid fee: {}",
                    style(&raw_amount).bold()
                ));
            }
        };

        // TODO: accept this as generic parameter instead
        let client = lib::http_api::HttpApi::new(endpoint);

        let spinner = SpinnerBuilder::new()
            .with_prefix(style("[1/4]").bold().dim())
            .with_text("fetching necessary data from the node")
            .start();

        let protocol_info = client.get_protocol_info().unwrap();
        let mut counter = client.get_counter_for_key(&from).unwrap();
        let constants = client.get_constants().unwrap();
        let head_block_hash = client.get_head_block_hash().unwrap();
        let manager_key = client.get_manager_key(&from).unwrap();

        spinner.finish();
        eprintln!(
            "{} {} {}",
            style("[1/4]").bold().green(),
            emojies::TICK,
            "fetched necessary data from the node",
        );

        let mut operation_group = NewOperationGroup::new(head_block_hash.clone());

        counter += 1;
        let tx_op = NewTransactionOperationBuilder::new()
            .source(from.clone())
            .destination(to.clone())
            .amount(amount.to_string())
            .fee(fee.to_string())
            .counter(counter.to_string())
            .gas_limit(50000.to_string())
            .storage_limit(constants.hard_storage_limit_per_operation.to_string())
            .build()
            .unwrap();
        operation_group = operation_group.with_transaction(tx_op);

        if manager_key.is_none() {
            counter += 1;
            let reveal_op = NewRevealOperationBuilder::new()
                .source(from.clone())
                .public_key(
                    PublicKey::from_base58check(
                        &trezor_execute(
                            trezor.as_mut().unwrap().get_public_key(key_path.as_ref().unwrap().clone())
                        )
                    ).unwrap()
                )
                .fee(fee.to_string())
                .counter(counter.to_string())
                .gas_limit(50000.to_string())
                .storage_limit(constants.hard_storage_limit_per_operation.to_string())
                .build()
                .unwrap();
            operation_group = operation_group.with_reveal(reveal_op);
        }

        let sig_info = {
            if !use_trezor {
                let _spinner = SpinnerBuilder::new()
                    .with_prefix(style("[2/4]").bold().dim())
                    .with_text("forging the operation and signing")
                    .start();
                let forged_operation = client.forge_operations(&head_block_hash, &operation_group).unwrap();

                let local_signer = {
                    let (pub_key, priv_key) = match get_keys_by_pkh(&from) {
                        Ok(keys) => keys,
                        Err(_) => {
                            exit_with_error(format!(
                                "no local wallet with public key hash: {}",
                                style(from.to_base58check()).bold()
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
                    "forging and signing transaction using Trezor",
                );
                let mut tx: TezosSignTx = operation_group.clone().into();
                tx.set_address_n(key_path.unwrap().clone());
                let result = OperationSignatureInfo::from(
                    trezor_execute(trezor.as_mut().unwrap().sign_tx(tx))
                );

                Term::stderr().clear_last_lines(1).unwrap();

                result
            }
        };
        let signature = sig_info.signature.clone();
        let operation_with_signature = sig_info.operation_with_signature.clone();
        let operation_hash = sig_info.operation_hash.clone();

        eprintln!(
            "{} {} {}",
            style("[2/4]").bold().green(),
            emojies::TICK,
            "operation forged and signed",
        );

        let spinner = SpinnerBuilder::new()
            .with_prefix(style("[3/4]").bold().dim())
            .with_text("applying and injecting the operation")
            .start();

        client.preapply_operations(
            &protocol_info.next_protocol_hash,
            &head_block_hash,
            &signature,
            &operation_group,
        ).unwrap();

        client.inject_operations(&operation_with_signature).unwrap();

        spinner.finish();
        eprintln!(
            "{} {} {}",
            style("[3/4]").bold().green(),
            emojies::TICK,
            "applied and injected the operation",
        );

        let spinner = SpinnerBuilder::new()
            .with_prefix(style("[4/4]").bold().dim())
            .with_text("waiting for confirmation")
            .start();

        for _ in 0..10 {
            thread::sleep(Duration::from_secs(2));

            let status = client.get_pending_operation_status(&operation_hash).unwrap();
            match status {
                PendingOperationStatus::Refused => {
                    exit_with_error("transaction refused");
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
        eprintln!();

        eprintln!(
            "  {}View operation at: {}/{}",
            emojies::FINGER_POINTER_RIGHT,
            style("https://delphinet.tezblock.io/transaction").cyan(),
            style(&operation_hash).cyan(),
        );

        if !console::user_attended() {
            println!("{}", &operation_hash);
        }
    }
}
