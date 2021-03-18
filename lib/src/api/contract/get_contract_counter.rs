use std::fmt::{self, Display};

use crate::{Address, ToBase58Check};
use crate::api::TransportError;

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub enum GetContractCounterErrorKind {
    Transport(#[from] TransportError),
    #[error("Unknown! {0}")]
    Unknown(String),
}

#[derive(thiserror::Error, Debug)]
pub struct GetContractCounterError {
    pub address: Address,
    pub kind: GetContractCounterErrorKind,
}

impl Display for GetContractCounterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "getting counter for address \"{}\" failed! Reason: {}",
            self.address.to_base58check(),
            self.kind,
        )
    }
}

pub type GetContractCounterResult = Result<u64, GetContractCounterError>;

pub trait GetContractCounter {
    /// Get counter for a contract.
    fn get_contract_counter(&self, address: &Address) -> GetContractCounterResult;
}
