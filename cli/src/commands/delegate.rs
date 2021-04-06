use structopt::StructOpt;

use crate::commands::CommandError;

use crate::common::operation_command::{RawOperationCommand, RawOptions};

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

    #[structopt(long = "ledger")]
    pub use_ledger: bool,

    /// Address to delegate tezos from.
    ///
    /// Can either be public key hash: tz1av5nBB8Jp6VZZDBdmGifRcETaYc7UkEnU
    ///
    /// Or if --trezor flag is set, key derivation path**, like: "m/44'/1729'/0'"
    #[structopt(short, long)]
    pub from: String,

    #[structopt(short, long)]
    pub to: String,

    /// Specify fee for the delegation.
    ///
    /// If not specified, fee will be estimated and you will be prompted
    /// whether or not you accept estimate or would like to enter custom one.
    #[structopt(long)]
    pub fee: Option<String>,
}

impl RawOperationCommand for Delegate {
    fn get_raw_options(&self) -> RawOptions {
        RawOptions {
            api_type: "http".to_string(),
            use_trezor: self.use_trezor,
            use_ledger: self.use_ledger,
        }
    }

    fn get_api_endpoint(&self) -> String {
        self.endpoint.clone()
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

impl Delegate {
    pub fn execute(self) -> Result<(), CommandError> {
        Ok(self.parse()?.delegate()?)
    }
}
