use structopt::StructOpt;

use crate::common::exit_with_error;
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

    /// E.g. m/44'/1729'/0'
    #[structopt(short, long)]
    path: String
}

impl GetAddress {
    pub fn execute(self) {
        let mut trezor = find_trezor_device()
            .connect()
            .unwrap();
        trezor.init_device().unwrap();

        // TODO: better parsing
        let path = self.path
            .replace("m/", "")
            .split("/")
            .map(|num| {
                let mut num = num.to_string();
                let is_hardened = num.ends_with("'");

                if is_hardened {
                    num.pop();
                }

                let num = match num.parse() {
                    Ok(n) => n,
                    Err(_) => exit_with_error("invalid path"),
                };
                if is_hardened {
                    num + 2147483648
                } else {
                    num
                }
            })
            .collect::<Vec<_>>();

        let address = trezor_execute(
            trezor.get_address(path.clone()),
        )
            .get_address()
            .to_string();

        println!("{}", address);
    }
}
