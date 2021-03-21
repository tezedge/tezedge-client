use std::fmt::{self, Display};

use crate::NewOperationGroup;
use crate::api::TransportError;

#[derive(thiserror::Error, Debug)]
pub enum PreapplyOperationsError {
    Transport(#[from] TransportError),
    Unknown(String),
}

impl Display for PreapplyOperationsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "preapplying operation failed! Reason: ")?;
        match self {
            Self::Transport(err) => err.fmt(f),
            Self::Unknown(err) => write!(f, "Unknown! {}", err)
        }
    }
}

pub type PreapplyOperationsResult = Result<serde_json::Value, PreapplyOperationsError>;

pub trait PreapplyOperations {
    fn preapply_operations(
        &self,
        operation_group: &NewOperationGroup,
        signature: &str,
    ) -> PreapplyOperationsResult;
}
