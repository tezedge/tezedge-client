use types::Address;
use crypto::ToBase58Check;
use crate::api::{
    GetContractDelegate, GetContractDelegateResult,
    TransportError, GetContractDelegateError, GetContractDelegateErrorKind,
};
use crate::http_api::HttpApi;

fn get_contract_delegate_url(base_url: &str, addr: &Address) -> String {
    format!(
        "{}/chains/main/blocks/head/context/contracts/{}/delegate",
        base_url,
        addr.to_base58check(),
    )
}

impl From<ureq::Error> for GetContractDelegateErrorKind {
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

impl From<std::io::Error> for GetContractDelegateErrorKind {
    fn from(error: std::io::Error) -> Self {
        Self::Transport(TransportError(Box::new(error)))
    }
}

#[inline]
fn build_error<E>(address: &Address, kind: E) -> GetContractDelegateError
    where E: Into<GetContractDelegateErrorKind>,
{
    GetContractDelegateError {
        address: address.clone(),
        kind: kind.into(),
    }
}

impl GetContractDelegate for HttpApi {
    fn get_contract_delegate(&self, addr: &Address) -> GetContractDelegateResult {
        Ok(self.client.get(&get_contract_delegate_url(&self.base_url, addr))
           .call()
           .map(|resp| resp.into_json())
           .or_else(|err| {
               match &err {
                   // will return 404 status if `addr` hasn't an active delegation.
                   ureq::Error::Status(404, _) => Ok(Ok(None)),
                   _ => Err(build_error(addr, err))
               }
           })?
           .map_err(|err| build_error(addr, err))?)
    }
}
