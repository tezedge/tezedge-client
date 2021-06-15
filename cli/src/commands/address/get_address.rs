use std::fmt::{self, Display};
use structopt::StructOpt;
use console::style;

use lib::{ToBase58Check, KeyDerivationPath};

use crate::commands::CommandError;

/// Get address from hardware wallet.
///
/// Outputs Public key hash to stdout.
#[derive(StructOpt, Debug, Clone)]
pub struct GetAddress {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Get address from Ledger.
    #[structopt(long)]
    trezor: bool,

    /// Get address from Ledger.
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
        let path: KeyDerivationPath = self.path.parse()?;

        let address = if self.trezor {
            crate::trezor::get_address(
                &mut crate::trezor::find_device_and_connect(),
                &path,
            ).to_base58check()
        } else if self.ledger {
            let mut ledger = crate::ledger::find_device_and_connect();

            crate::ledger::ledger_execute(
                ledger.get_address(&path, false)
            )
                .to_base58check()
        } else {
            Err(NoSourceSpecifiedError)?
        };

        println!("{}", address);
        Ok(())
    }
}
