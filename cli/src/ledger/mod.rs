mod execute;
pub use execute::*;

use lib::ledger_api::Ledger;

use crate::common::exit_with_error;

pub fn find_device_and_connect() -> Ledger {
    match Ledger::connect() {
        Ok(x) => x,
        Err(err) => exit_with_error(err),
    }
}
