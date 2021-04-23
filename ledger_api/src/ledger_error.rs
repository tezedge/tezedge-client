use ledger_apdu::map_apdu_error_description;
use ledger::LedgerHIDError;

/// Ledger Error
#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub enum LedgerError {
    Transport(#[from] LedgerHIDError),
    #[error("{}", map_apdu_error_description(*.0))]
    APDU(u16),
    RunApp(#[from] RunAppError),
    #[error("invalid data length received from ledger")]
    InvalidDataLength,
    #[error("ledger unlock request timed out")]
    UnlockRequestTimeout,
}

impl LedgerError {
    /// Whether Ledger needs to be reconnected to.
    ///
    /// When opening an application on the device, Ledger disconnects
    /// and needs to be reconnected.
    pub fn needs_reconnect(&self) -> bool {
        let transport_err = match self {
            Self::Transport(err) => err,
            _ => { return false; }
        };

        match &transport_err {
            LedgerHIDError::Hid(hid_err) => {
                match &hid_err {
                    hidapi::HidError::HidApiError { message } => {
                        if message == "No such device" {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        false
    }

    /// Whether Ledger needs unlock or not.
    ///
    /// If Ledger gets locked while Tezos app is open, instead
    /// of waiting for unlock, it sends error `0x6804` which needs
    /// special treatment.
    pub fn needs_unlock(&self) -> bool {
        matches!(self, Self::APDU(0x6804))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RunAppErrorKind {
    #[error("{0}")]
    Transport(#[from] LedgerHIDError),
    #[error("app name is too large! Length must be <= 255")]
    NameTooLarge,
    #[error("{}", map_apdu_error_description(*.0))]
    APDU(u16),
    #[error("failed to reconnect after opening an app. {0}")]
    Reconnect(LedgerHIDError),
}

#[derive(thiserror::Error, Debug)]
#[error("running app with name \"{name}\" on ledger failed! Reason: {kind}")]
pub struct RunAppError {
    pub name: String,
    pub kind: RunAppErrorKind,
}
