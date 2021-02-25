mod get_pkh;
pub use get_pkh::*;

mod execute;
pub use execute::*;

use lib::trezor_api;
use trezor_api::Trezor;

use crate::common::exit_with_error;

pub fn find_device() -> trezor_api::AvailableDevice {
    let mut devices = trezor_api::find_devices().unwrap();

    // TODO: only allow trezor T
    match devices.len() {
        // TODO: show message that device is not conntected and wait
        // till it's connected.
        0 => exit_with_error("Trezor not connected"),
        1 => devices.remove(0),
        // TODO: show select with filtering to choose between devices
        _ => exit_with_error("More than one Trezor connected (unsupported for now)"),
    }
}

pub fn find_device_and_connect() -> Trezor {
    let mut trezor = find_device()
        .connect()
        .unwrap();
    trezor.init_device().unwrap();
    trezor
}
