use lib::trezor_api::{Result, TrezorResponse};
use lib::trezor_api::messages::TrezorMessage;
use cli_spinner::wait_for_action_spinner;
use crate::common::exit_with_error;

/// Execute Trezor command and drive it to completion.
///
/// Handles action requests and returns successful result `T` at the end,
/// if no error occurred. Otherwise [crate::common::exit_with_error] will
/// be called, which will print an error to `stderr` and will exit the process
/// with code 1: `process.exit(1)`.
///
/// Warning: At the moment if before calling `ack` or before response comes for `ack`,
///          process crashes or is terminated for some reason, Trezor seems to get stuck.
// TODO: Explore the issue and fix it or at least print useful error message, when
//       the case detailed in the 'Warning', happens.
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
