use structopt::StructOpt;

use crate::common::{exit_with_error, parse_derivation_path};
use crate::trezor::{find_trezor_device, trezor_execute};

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
    pub fn execute(self) {
        let mut trezor = find_trezor_device()
            .connect()
            .unwrap();
        trezor.init_device().unwrap();

        let path = parse_derivation_path(&self.path);

        let address = trezor_execute(
            trezor.get_address(path.clone()),
        );

        println!("{}", address);
    }
}
