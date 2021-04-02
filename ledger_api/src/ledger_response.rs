use crate::{LedgerError, RetryRequest, ReconnectRequest, RunAppRequest};

pub enum LedgerResponse<'a, T> {
    Ok(T),
    Err(LedgerError),
    RetryRequest(RetryRequest<'a, T>),
    ReconnectRequest(ReconnectRequest<'a, T>),
    RunAppRequest(RunAppRequest<'a, T>),
}

impl<'a, T> From<Result<T, LedgerResponse<'a, T>>> for LedgerResponse<'a, T> {
    fn from(res: Result<T, LedgerResponse<'a, T>>) -> Self {
        match res {
            Ok(x) => LedgerResponse::Ok(x),
            Err(resp) => resp,
        }
    }
}

impl<'a, T> From<Result<T, LedgerError>> for LedgerResponse<'a, T> {
    fn from(res: Result<T, LedgerError>) -> Self {
        match res {
            Ok(x) => LedgerResponse::Ok(x),
            Err(resp) => LedgerResponse::Err(resp),
        }
    }
}

impl<'a, T> From<LedgerError> for LedgerResponse<'a, T> {
    fn from(err: LedgerError) -> Self {
        LedgerResponse::Err(err)
    }
}

impl<'a, T> From<RetryRequest<'a, T>> for LedgerResponse<'a, T> {
    fn from(req: RetryRequest<'a, T>) -> Self {
        LedgerResponse::RetryRequest(req)
    }
}

impl<'a, T> From<ReconnectRequest<'a, T>> for LedgerResponse<'a, T> {
    fn from(req: ReconnectRequest<'a, T>) -> Self {
        LedgerResponse::ReconnectRequest(req)
    }
}

impl<'a, T> From<RunAppRequest<'a, T>> for LedgerResponse<'a, T> {
    fn from(req: RunAppRequest<'a, T>) -> Self {
        LedgerResponse::RunAppRequest(req)
    }
}
