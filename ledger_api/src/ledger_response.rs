use crate::{LedgerError, RetryRequest, ReconnectRequest, UnlockRequest, RunAppRequest};

pub enum LedgerResponse<'a, T> {
    Ok(T),
    Err(LedgerError),
    RetryRequest(RetryRequest<'a, T>),
    ReconnectRequest(ReconnectRequest<'a, T>),
    UnlockRequest(UnlockRequest<'a, T>),
    RunAppRequest(RunAppRequest<'a, T>),
}

impl<'a, T> LedgerResponse<'a, T> {
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }

    pub fn is_err(&self) -> bool {
        matches!(self, Self::Err(_))
    }
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

impl<'a, T> From<UnlockRequest<'a, T>> for LedgerResponse<'a, T> {
    fn from(req: UnlockRequest<'a, T>) -> Self {
        LedgerResponse::UnlockRequest(req)
    }
}

impl<'a, T> From<RunAppRequest<'a, T>> for LedgerResponse<'a, T> {
    fn from(req: RunAppRequest<'a, T>) -> Self {
        LedgerResponse::RunAppRequest(req)
    }
}
