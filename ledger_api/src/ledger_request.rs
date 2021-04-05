use ledger_apdu::APDUCommand;

use crate::{Ledger, LedgerError, LedgerResponse};

pub type ResultHandler<T> = dyn Fn(&mut Ledger, Vec<u8>) -> Result<T, LedgerError>;

pub struct LedgerRequestData<T> {
    pub command: APDUCommand,
    pub handler: Box<ResultHandler<T>>,
}

impl<T> LedgerRequestData<T>
    where T: 'static,
{
    pub fn map<U, F>(self, handler: F) -> LedgerRequestData<U>
        where F: 'static + Fn(&mut Ledger, T) -> Result<U, LedgerError>,
              U: 'static,
    {
        let old_handler = self.handler;
        LedgerRequestData {
            command: self.command,
            handler: Box::new(move |ledger, raw_result| {
                let res = old_handler(ledger, raw_result)?;
                handler(ledger, res)
            }),
        }
    }

    pub fn send<'a>(self, ledger: &'a mut Ledger) -> LedgerResponse<'a, T>
        where T: 'static,
    {
        ledger.call(self.command, self.handler).into()
    }
}

pub struct LedgerRequest<'a, T>
{
    pub ledger: &'a mut Ledger,
    pub data: LedgerRequestData<T>,
}

impl<'a> LedgerRequest<'a, Vec<u8>> {
    pub fn new(
        ledger: &'a mut Ledger,
        command: APDUCommand,
    ) -> Self {
        LedgerRequest {
            ledger,
            data: LedgerRequestData {
                command,
                handler: Box::new(|_, x| Ok(x)),
            },
        }
    }
}

impl<'a, T: 'a> LedgerRequest<'a, T>
    where T: 'static,
{
    pub fn map<U, F>(self, handler: F) -> LedgerRequest<'a, U>
        where F: 'static + Fn(&mut Ledger, T) -> Result<U, LedgerError>,
              U: 'static,
    {
        LedgerRequest {
            ledger: self.ledger,
            data: self.data.map(handler),
        }
    }

    pub fn send(self) -> LedgerResponse<'a, T>
        where T: 'static,
    {
        self.data.send(self.ledger)
    }
}
