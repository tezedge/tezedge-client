use crate::{Address, PublicKey, ToBase58Check, FromPrefixedBase58CheckError};
use crate::api::{
    GetManagerPublicKey, GetManagerPublicKeyResult,
    TransportError, GetManagerPublicKeyError, GetManagerPublicKeyErrorKind,
};
use super::HttpApi;

/// Get manager key
fn get_manager_key_url(base_url: &str, addr: &Address) -> String {
    format!(
        "{}/chains/main/blocks/head/context/contracts/{}/manager_key",
        base_url,
        addr.to_base58check(),
    )
}

impl From<ureq::Error> for GetManagerPublicKeyErrorKind {
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

impl From<std::io::Error> for GetManagerPublicKeyErrorKind {
    fn from(error: std::io::Error) -> Self {
        Self::Transport(TransportError(Box::new(error)))
    }
}

#[inline]
fn build_error<E>(address: &Address, error: E) -> GetManagerPublicKeyError
    where E: Into<GetManagerPublicKeyErrorKind>,
{
    GetManagerPublicKeyError {
        address: address.clone(),
        error: error.into(),
    }
}

// TODO: receiving NULL, probably because node isn't synced
impl GetManagerPublicKey for HttpApi {
    fn get_manager_public_key(&self, addr: &Address) -> GetManagerPublicKeyResult {
        Ok(self.client.get(&get_manager_key_url(&self.base_url, addr))
           .call()
           .map_err(|err| build_error(addr, err))?
           .into_json::<Option<String>>()
           .map_err(|err| build_error(addr, err))?
           .map(|key| PublicKey::from_base58check(&key))
           .transpose()
           .map_err(|err| build_error(addr, err))?)
    }
}

