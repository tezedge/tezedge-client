use console::style;

use lib::ledger_api::LedgerResponse;
use cli_spinner::{wait_for_action_spinner, SpinnerBuilder};
use crate::common::exit_with_error;

/// Execute Ledger command and drive it to completion.
///
/// Handles action requests and returns successful result `T` at the end,
/// if no error occurred. Otherwise [crate::common::exit_with_error] will
/// be called, which will print an error to `stderr` and will exit the process
/// with code 1: `process.exit(1)`.
pub fn ledger_execute<'a, T>(mut response: LedgerResponse<'a, T>) -> T
    where T: 'static,
{
    let spinner = wait_for_action_spinner();

    loop {
        match response {
            LedgerResponse::Ok(result) => { return result; }
            LedgerResponse::Err(err) => {
                exit_with_error(err);
            }
            LedgerResponse::RunAppRequest(req) => {
                let _spinner = spinner.clone()
                    .with_text(format!(
                        "please open {} on your Ledger device to proceed.",
                        style(req.app_name()).green(),
                    ))
                    .start();
                response = req.ack();
            }
            LedgerResponse::RetryRequest(req) => {
                let _spinner = SpinnerBuilder::new()
                    .with_text("retrying request.")
                    .start();
                response = req.ack();
            }
            LedgerResponse::UnlockRequest(req) => {
                let _spinner = spinner.clone()
                    .with_text(format!(
                        "please {} your Ledger device.",
                        style("unlock").bold(),
                    ))
                    .start();
                response = req.ack();
            }
            LedgerResponse::ReconnectRequest(req) => {
                let _spinner = SpinnerBuilder::new()
                    .with_text(format!(
                        "{} Trying to reconnect.",
                        style("Ledger disconnected!").red(),
                    ))
                    .start();
                response = req.ack();
            }
        }
    }
}
