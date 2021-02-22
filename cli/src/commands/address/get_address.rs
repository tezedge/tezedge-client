use structopt::StructOpt;
use console::style;

use lib::trezor_api;

use crate::common::exit_with_error;

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

fn find_trezor_device() -> trezor_api::AvailableDevice {
    let mut devices = trezor_api::find_devices().unwrap();

    // TODO: only allow trezor T
    match devices.len() {
        0 => exit_with_error("Trezor not connected"),
        1 => devices.remove(0),
        // TODO: show select with filtering to choose between devices
        _ => exit_with_error("More than one Trezor connected (unsupported for now)"),
    }
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

        println!("{}", trezor.get_address(path).unwrap().ok().unwrap().get_address());
    }
}
