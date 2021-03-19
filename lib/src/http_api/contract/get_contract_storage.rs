use crate::{OriginatedAddress, ToBase58Check};
use crate::api::{
    GetContractStorage, GetContractStorageResult,
    TransportError, GetContractStorageError, GetContractStorageErrorKind,
};
use crate::http_api::HttpApi;

/// Get manager key
fn get_contract_storage_url(base_url: &str, addr: &OriginatedAddress) -> String {
    format!(
        "{}/chains/main/blocks/head/context/contracts/{}/storage",
        base_url,
        addr.to_base58check(),
    )
}

impl From<ureq::Error> for GetContractStorageErrorKind {
    fn from(error: ureq::Error) -> Self {
        match error {
            ureq::Error::Transport(error) => {
                Self::Transport(TransportError(Box::new(error)))
            }
            ureq::Error::Status(code, resp) => {
                let status_text = resp.status_text().to_string();
                Self::Unknown(format!(
                    "Http status: ({}, {}){}",
                    code,
                    status_text,
                    match resp.into_string() {
                        Ok(s) => format!(", message: {}", s),
                        Err(_) => "".to_string(),
                    },
                ))
            }
        }
    }
}

impl From<std::io::Error> for GetContractStorageErrorKind {
    fn from(error: std::io::Error) -> Self {
        Self::Transport(TransportError(Box::new(error)))
    }
}

#[inline]
fn build_error<E>(address: &OriginatedAddress, kind: E) -> GetContractStorageError
    where E: Into<GetContractStorageErrorKind>,
{
    GetContractStorageError {
        address: address.clone(),
        kind: kind.into(),
    }
}

impl GetContractStorage for HttpApi {
    fn get_contract_storage(
        &self,
        addr: &OriginatedAddress,
    ) -> GetContractStorageResult
    {
        Ok(self.client.get(&get_contract_storage_url(&self.base_url, addr))
           .call()
           .map_err(|err| build_error(addr, err))?
           .into_json()
           .map_err(|err| build_error(addr, err))?)
    }
}
