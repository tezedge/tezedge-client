use structopt::StructOpt;
use console::style;

use lib::utils::parse_float_amount;
use crate::commands::CommandError;
use crate::common::exit_with_error;
use crate::common::operation_command::{RawOperationCommand, RawOptions};

/// Create a transaction
///
/// Outputs transaction hash to stdout in case of success.
#[derive(StructOpt)]
pub struct Transfer {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    /// Disable interactivity and accept default answers to prompts.
    #[structopt(short = "y", long = "no-prompt")]
    pub no_prompt: bool,

    #[structopt(short = "E", long)]
    pub endpoint: String,

    #[structopt(long = "trezor")]
    pub use_trezor: bool,

    #[structopt(long = "ledger")]
    pub use_ledger: bool,

    /// Required only if `--from` argument is an originated
    /// (starts with KT1) address.
    #[structopt(long = "key-path")]
    pub key_path: Option<String>,

    /// Address to transfer tezos from.
    ///
    /// When transfering from originated (KT1) address, this needs to be
    /// that KT1 address.
    ///
    /// Otherwise use key derivation path, like: "m/44'/1729'/0'/0'"
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
}

impl RawOperationCommand for Transfer {
    fn get_raw_options(&self) -> RawOptions {
        RawOptions {
            api_type: "http".to_string(),
            no_prompt: self.no_prompt,
            use_trezor: self.use_trezor,
            use_ledger: self.use_ledger,
        }
    }

    fn get_api_endpoint(&self) -> String {
        self.endpoint.clone()
    }

    fn get_raw_key_path(&self) -> Option<&str> {
        self.key_path.as_ref().map(|s| s.as_str())
    }

    fn get_raw_from(&self) -> &str {
        &self.from
    }

    fn get_raw_to(&self) -> &str {
        &self.to
    }

    fn get_raw_fee(&self) -> Option<&String> {
        self.fee.as_ref()
    }
}

impl Transfer {
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

    pub fn execute(self) -> Result<(), CommandError> {
        Ok(self.parse()?.transfer(self.get_amount())?)
    }
}
