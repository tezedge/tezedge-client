use std::fmt::{self, Display};

use types::OriginatedAddress;
use crypto::ToBase58Check;
use crate::api::TransportError;

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub enum GetContractStorageErrorKind {
    Transport(#[from] TransportError),
    #[error("Unknown! {0}")]
    Unknown(String),
}

#[derive(thiserror::Error, Debug)]
pub struct GetContractStorageError {
    pub address: OriginatedAddress,
    pub kind: GetContractStorageErrorKind,
}

impl Display for GetContractStorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "getting storage for contract with an address \"{}\" failed! Reason: {}",
            self.address.to_base58check(),
            self.kind,
        )
    }
}

pub type GetContractStorageResult = Result<serde_json::Value, GetContractStorageError>;

pub trait GetContractStorage {
    fn get_contract_storage(
        &self,
        addr: &OriginatedAddress,
    ) -> GetContractStorageResult;
}
