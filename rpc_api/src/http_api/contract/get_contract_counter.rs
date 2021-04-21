use serde::Deserialize;

use types::ImplicitAddress;
use crypto::ToBase58Check;
use crate::api::{
    GetContractCounter, GetContractCounterResult,
    TransportError, GetContractCounterError, GetContractCounterErrorKind,
};
use crate::http_api::HttpApi;

fn get_contract_counter_url(base_url: &str, addr: &ImplicitAddress) -> String {
    format!(
        "{}/chains/main/blocks/head/context/contracts/{}/counter",
        base_url,
        addr.to_base58check(),
    )
}

impl From<ureq::Error> for GetContractCounterErrorKind {
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

impl From<std::io::Error> for GetContractCounterErrorKind {
    fn from(error: std::io::Error) -> Self {
        Self::Transport(TransportError(Box::new(error)))
    }
}

#[derive(Deserialize)]
#[serde(transparent)]
struct ContractCounter {
    #[serde(with = "utils::serde_str")]
    current: u64,
}

impl Into<u64> for ContractCounter {
    fn into(self) -> u64 {
        self.current
    }
}

#[inline]
fn build_error<E>(address: &ImplicitAddress, kind: E) -> GetContractCounterError
    where E: Into<GetContractCounterErrorKind>,
{
    GetContractCounterError {
        address: address.clone(),
        kind: kind.into(),
    }
}

// TODO: receiving NULL, probably because node isn't synced
impl GetContractCounter for HttpApi {
    fn get_contract_counter(&self, addr: &ImplicitAddress) -> GetContractCounterResult {
        Ok(self.client.get(&get_contract_counter_url(&self.base_url, addr))
           .call()
           .map_err(|err| build_error(addr, err))?
           .into_json::<ContractCounter>()
           .map_err(|err| build_error(addr, err))?
           .into())
    }
}
