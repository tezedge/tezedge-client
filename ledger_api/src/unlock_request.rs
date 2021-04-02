use std::time::Duration;

use crate::{Ledger, LedgerError, LedgerResponse};
use crate::ledger_request::LedgerRequestData;

/// Unlock request timeout in milliseconds. 2 minutes.
const UNLOCK_TIMEOUT_MILLIS: u128 = 120000;

/// Ledger is Locked. We need to wait for user to unlock it.
pub struct UnlockRequest<'a, T> {
    pub ledger: &'a mut Ledger,
    pub data: LedgerRequestData<T>,
}

impl<'a, T: 'a> UnlockRequest<'a, T>
    where T: 'static,
{
    pub fn new(ledger: &'a mut Ledger, data: LedgerRequestData<T>) -> Self {
        Self { ledger, data }
    }

    /// Wait for user to unlock the device.
    ///
    /// Will block until user unlocks the device.
    ///
    /// Timeout: [UNLOCK_TIMEOUT_MILLIS].
    pub fn ack(self) -> LedgerResponse<'a, T> {
        let sleep_duration = Duration::from_millis(500);
        let max_attempts = UNLOCK_TIMEOUT_MILLIS / sleep_duration.as_millis();

        std::thread::sleep(sleep_duration);
        let mut result = self.data.send(self.ledger);

        for _ in 1..max_attempts {
            if let LedgerResponse::UnlockRequest(req) = result {
                std::thread::sleep(sleep_duration);
                result = req.data.send(req.ledger);
            } else {
                return result;
            }
        }

        LedgerError::UnlockRequestTimeout.into()
    }
}
