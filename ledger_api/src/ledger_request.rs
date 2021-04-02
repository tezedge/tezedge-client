use ledger_apdu::APDUCommand;

use crate::{Ledger, LedgerError, LedgerResponse};

pub type ResultHandler<T> = dyn Fn(Vec<u8>) -> Result<T, LedgerError>;

pub struct LedgerRequestData<T> {
    pub command: APDUCommand,
    pub handler: Box<ResultHandler<T>>,
}

impl<T> LedgerRequestData<T>
    where T: 'static,
{
    pub fn map<'a, U, F>(self, handler: F) -> LedgerRequestData<U>
        where F: 'static + Fn(T) -> Result<U, LedgerError>,
              U: 'static,
    {
        let old_handler = self.handler;
        LedgerRequestData {
            command: self.command,
            handler: Box::new(move |raw_result| {
                let res = old_handler(raw_result)?;
                handler(res)
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
                handler: Box::new(|x| Ok(x)),
            },
        }
    }
}

impl<'a, T: 'a> LedgerRequest<'a, T>
    where T: 'static,
{
    pub fn map<U, F>(self, handler: F) -> LedgerRequest<'a, U>
        where F: 'static + Fn(T) -> Result<U, LedgerError>,
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
