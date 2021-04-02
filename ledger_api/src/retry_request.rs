use crate::{Ledger, LedgerResponse};
use crate::ledger_request::LedgerRequestData;

pub struct RetryRequest<'a, T> {
    pub ledger: &'a mut Ledger,
    pub data: LedgerRequestData<T>,
}

impl<'a, T: 'a> RetryRequest<'a, T>
    where T: 'static,
{
    pub fn new(ledger: &'a mut Ledger, data: LedgerRequestData<T>) -> Self {
        Self { ledger, data }
    }

    pub fn ack(self) -> LedgerResponse<'a, T> {
        self.data.send(self.ledger)
    }
}
