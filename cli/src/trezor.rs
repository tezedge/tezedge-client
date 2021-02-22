use console::style;

use lib::trezor_api;
pub use trezor_api::{Result, TrezorResponse};
pub use trezor_api::messages::TrezorMessage;

use crate::common::exit_with_error;
use crate::spinner::SpinnerBuilder;

pub fn find_trezor_device() -> trezor_api::AvailableDevice {
    let mut devices = trezor_api::find_devices().unwrap();

    // TODO: only allow trezor T
    match devices.len() {
        0 => exit_with_error("Trezor not connected"),
        1 => devices.remove(0),
        // TODO: show select with filtering to choose between devices
        _ => exit_with_error("More than one Trezor connected (unsupported for now)"),
    }
}

// TODO: at the moment if before calling "ack" or before response comes for "ack",
// process crashes or is terminated for some reason, Trezor seems to get stuck.
// Explore the issue and fix it or at least print useful error message.
pub fn trezor_execute<T, R>(mut response: Result<TrezorResponse<T, R>>) -> T
    where R: TrezorMessage,
{
    let spinner = SpinnerBuilder::new()
        .with_spinner_chars(vec![
            style("   ").red(),
            style(">  ").red(),
            style(">> ").red(),
            style(">>>").red(),
        ])
        .with_interval_ms(300);

    loop {
        // TODO: handle transport errors
        match response.unwrap() {
            TrezorResponse::Ok(result) => { return result; }
            TrezorResponse::ButtonRequest(req) => {
                let _spinner = spinner.clone()
                    .with_text("please confirm an action on Trezor device")
                    .start();
                response = req.ack();
            }
            TrezorResponse::Failure(failure) => {
                exit_with_error(failure.get_message());
            }
            TrezorResponse::PinMatrixRequest(req) => {
                // TODO: zeroize pin
                let pin = dialoguer::Password::new()
                    .with_prompt("Trezor pin code")
                    .interact()
                    .unwrap();
                response = req.ack_pin(pin);
            }
            TrezorResponse::PassphraseRequest(req) => {
                let _spinner = spinner.clone()
                    .with_text("please enter the passphase on Trezor device")
                    .start();
                response = req.ack();
            }
            TrezorResponse::PassphraseStateRequest(req) => {
                // TODO: revisit
                response = req.ack();
            }
        }
    }
}
