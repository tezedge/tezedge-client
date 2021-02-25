use lib::trezor_api::{Result, TrezorResponse};
use lib::trezor_api::messages::TrezorMessage;
use crate::spinner::wait_for_action_spinner;
use crate::common::exit_with_error;

// TODO: at the moment if before calling "ack" or before response comes for "ack",
// process crashes or is terminated for some reason, Trezor seems to get stuck.
// Explore the issue and fix it or at least print useful error message.
pub fn trezor_execute<T, R>(mut response: Result<TrezorResponse<T, R>>) -> T
    where R: TrezorMessage,
{
    let spinner = wait_for_action_spinner();

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
        }
    }
}
