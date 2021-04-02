use console::style;

use lib::ledger_api::LedgerResponse;
use crate::spinner::{wait_for_action_spinner, SpinnerBuilder};
use crate::common::exit_with_error;

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
