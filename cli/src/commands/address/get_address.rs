use std::fmt::{self, Display};
use structopt::StructOpt;
use console::style;

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

    #[structopt(long)]
    trezor: bool,

    #[structopt(long)]
    ledger: bool,

    /// E.g. "m/44'/1729'/0'"
    #[structopt(short, long)]
    path: String,
}

#[derive(thiserror::Error, Debug)]
pub struct NoSourceSpecifiedError;

impl Display for NoSourceSpecifiedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source was not specified.\n\n")?;
        write!(f,
            "{}: please specify source from which you want to get an address.\n",
            style("help").yellow(),
        )?;
        write!(f, "    - {} to get from Trezor.\n", style("--trezor").bold())?;
        write!(f, "    - {} to get from Ledger.\n", style("--ledger").bold())
    }
}

impl GetAddress {
    pub fn execute(self) -> Result<(), CommandError> {
        let path = parse_derivation_path(&self.path)?;

        let address = if self.trezor {
            crate::trezor::get_address(
                &mut crate::trezor::find_device_and_connect(),
                path,
            ).to_base58check()
        } else if self.ledger {
            lib::ledger_api::Ledger::connect()?
                .get_address(path, false)?
                .to_base58check()
        } else {
            Err(NoSourceSpecifiedError)?
        };

        println!("{}", address);
        Ok(())
    }
}
