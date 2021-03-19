use std::fmt::{self, Display};

use crate::NewOperationGroup;
use crate::api::{TransportError, GetChainIDError};

#[derive(thiserror::Error, Debug)]
pub enum RunOperationError {
    Transport(#[from] TransportError),
    GetChainID(#[from] GetChainIDError),
    Unknown(String),
}

impl Display for RunOperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "operation simulation failed! Reason: ")?;
        match self {
            Self::Transport(err) => err.fmt(f),
            Self::GetChainID(err) => write!(f, "\n{}", err),
            Self::Unknown(err) => write!(f, "Unknown! {}", err),
        }
    }
}

pub type RunOperationResult = Result<serde_json::Value, RunOperationError>;

pub trait RunOperation {
    /// Simulate an operation.
    ///
    /// Useful for calculating fees as is returns estimated consumed gas,
    /// and it doesn't require signing the operation first.
    fn run_operation(
        &self,
        operation_group: &NewOperationGroup,
    ) -> RunOperationResult;
}
