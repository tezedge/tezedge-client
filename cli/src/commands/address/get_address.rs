use structopt::StructOpt;

use lib::ToBase58Check;

use crate::common::parse_derivation_path;
use crate::commands::CommandError;

/// Get address
///
/// Outputs transaction hash to stdout in case of success.
#[derive(StructOpt, Debug, Clone)]
pub struct GetAddress {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    #[structopt(short = "T", long)]
    trezor: bool,

    /// E.g. "m/44'/1729'/0'"
    #[structopt(short, long)]
    path: String
}

impl GetAddress {
    pub fn execute(self) -> Result<(), CommandError> {
        let path = parse_derivation_path(&self.path)?;
        let address = crate::trezor::get_address(
            &mut crate::trezor::find_device_and_connect(),
            path,
        ).to_base58check();

        println!("{}", address);
        Ok(())
    }
}
