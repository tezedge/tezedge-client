use std::{process, thread};
use std::fmt::Display;
use std::time::Duration;
use structopt::StructOpt;
use console::{style, Term};

use lib::api::*;
use lib::{PublicKey, PrivateKey};
use lib::utils::parse_float_amount;
use lib::signer::{SignOperation, LocalSigner};

use crate::options::Options;
use crate::spinner::SpinnerBuilder;
use crate::emojies;

/// Create a transaction
///
/// Outputs transaction hash to stdout in case of success.
#[derive(StructOpt, Debug, Clone)]
pub struct Transfer {
    #[structopt(short, long)]
    from: String,
    #[structopt(short, long)]
    to: String,
    #[structopt(short, long)]
    amount: String,
}

// TODO: replace with query to persistent encrypted store for keys
fn get_keys_by_pkh(pkh: &str) -> Result<(PublicKey, PrivateKey), ()> {
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

fn exit_with_error<E: Display>(error: E) -> ! {
    eprintln!(
        "{} {}",
        style("[ERROR]").red().bold(),
        error,
    );
    process::exit(1)
}

impl Transfer {
    pub fn execute(self, global_options: Options) {
        let from = self.from;
        let to = self.to;
        let amount = match parse_float_amount(&self.amount) {
            Ok(amount) => amount,
            Err(_) => {
                exit_with_error(format!(
                    "invalid amount: {}",
                    style(&self.amount).bold()
                ));
            }
        };

        let local_signer = {
            let (pub_key, priv_key) = match get_keys_by_pkh(&from) {
                Ok(keys) => keys,
                Err(_) => {
                    exit_with_error(format!(
                        "no local wallet with public key hash: {}",
                        style(&self.amount).bold()
                    ));
                }
            };
            LocalSigner::new(pub_key, priv_key)
        };

        // TODO: accept this as generic parameter instead
        let client = lib::http_api::HttpApi::new(global_options.endpoint);

        let spinner = SpinnerBuilder::new()
            .with_prefix(style("[1/4]").bold().dim())
            .with_text("fetching necessary data from the node")
            .start();

        let protocol_info = client.get_protocol_info().unwrap();
        let counter = client.get_counter_for_key(&from).unwrap() + 1;
        let constants = client.get_constants().unwrap();
        let head_block_hash = client.get_head_block_hash().unwrap();

        spinner.finish();
        eprintln!(
            "{} {} {}",
            style("[1/4]").bold().green(),
            emojies::TICK,
            "fetched necessary data from the node",
        );

        let tx = TransactionOperationBuilder::new()
            .source(from.to_string())
            .destination(to.to_string())
            .amount(amount.to_string())
            .fee("100000".to_string())
            .counter(counter.to_string())
            .gas_limit(50000.to_string())
            .storage_limit(constants.hard_storage_limit_per_operation.to_string())
            .build()
            .unwrap();


        let spinner = SpinnerBuilder::new()
            .with_prefix(style("[2/4]").bold().dim())
            .with_text("forging the operation and signing")
            .start();

        let operations = &[tx.into()];
        let forged_operation = client.forge_operations(&head_block_hash, operations).unwrap();

        let sig_info = local_signer.sign_operation(forged_operation.clone()).unwrap();
        let signature = sig_info.signature.clone();
        let operation_with_signature = sig_info.operation_with_signature.clone();
        let operation_hash = sig_info.operation_hash.clone();

        spinner.finish();
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
            operations,
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
                _ => {}
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
