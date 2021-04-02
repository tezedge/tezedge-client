use crate::{Ledger, LedgerResponse, RetryRequest};
use crate::ledger_request::LedgerRequestData;

/// Tezos app is closed and to continue, it needs to be ran.
pub struct RunAppRequest<'a, T> {
    ledger: &'a mut Ledger,
    initial_command: LedgerRequestData<T>,
    app_name: &'a str,
}

impl<'a, T: 'a> RunAppRequest<'a, T>
    where T: 'static,
{
    pub fn new(
        ledger: &'a mut Ledger,
        initial_command: LedgerRequestData<T>,
        app_name: &'a str,
    ) -> Self
    {
        Self { ledger, initial_command, app_name }
    }

    pub fn app_name(&self) -> &'a str {
        self.app_name
    }

    pub fn ack(self) -> LedgerResponse<'a, T> {
        match self.ledger.run_app(self.app_name) {
            // now that the app is open, retry initial command.
            Ok(_) => RetryRequest::new(self.ledger, self.initial_command).into(),
            Err(err) => LedgerResponse::Err(err.into()),
        }
    }
}
