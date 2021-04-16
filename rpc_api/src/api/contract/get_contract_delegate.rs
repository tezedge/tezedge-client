use std::fmt::{self, Display};

use types::{Address, ImplicitAddress};
use crypto::ToBase58Check;
use crate::api::TransportError;

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub enum GetContractDelegateErrorKind {
    Transport(#[from] TransportError),
    #[error("Unknown! {0}")]
    Unknown(String),
}

#[derive(thiserror::Error, Debug)]
pub struct GetContractDelegateError {
    pub address: Address,
    pub kind: GetContractDelegateErrorKind,
}

impl Display for GetContractDelegateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "getting delegate for address \"{}\" failed! Reason: {}",
            self.address.to_base58check(),
            self.kind,
        )
    }
}

pub type GetContractDelegateResult = Result<Option<ImplicitAddress>, GetContractDelegateError>;

pub trait GetContractDelegate {
    /// Get active delegate for a contract.
    fn get_contract_delegate(&self, address: &Address) -> GetContractDelegateResult;
}
