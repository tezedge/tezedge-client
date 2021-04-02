use crate::{Ledger, LedgerResponse, RetryRequest};
use crate::ledger_request::LedgerRequestData;

pub struct ReconnectRequest<'a, T> {
    pub ledger: &'a mut Ledger,
    pub data: LedgerRequestData<T>,
}

impl<'a, T: 'a> ReconnectRequest<'a, T>
    where T: 'static,
{
    pub fn new(ledger: &'a mut Ledger, data: LedgerRequestData<T>) -> Self {
        Self { ledger, data }
    }

    pub fn ack(self) -> LedgerResponse<'a, T> {
        if let Err(err) = self.ledger.reconnect(10) {
            return LedgerResponse::Err(err.into());
        }
        
        // retry request after reconnection succeeds
        RetryRequest::new(self.ledger, self.data).into()
    }
}
