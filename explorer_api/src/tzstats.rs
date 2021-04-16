use std::convert::TryFrom;
use std::error::Error;
use serde::{Serialize, Deserialize};

use types::{Network, ImplicitAddress};
use crate::UnsupportedNetworkError;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Baker {
    pub address: ImplicitAddress,
}

enum SupportedNetwork {
    Main(String),
    Beta(String),
    Florence(String),
    Edo(String),
    Delphi(String),
}

impl TryFrom<Network> for SupportedNetwork {
    type Error = UnsupportedNetworkError;

    fn try_from(network: Network) -> Result<Self, Self::Error> {
        match network {
            Network::Main(s) => Ok(SupportedNetwork::Main(s)),
            Network::Beta(s) => Ok(SupportedNetwork::Beta(s)),
            Network::Florence(s) => Ok(SupportedNetwork::Florence(s)),
            Network::Edo(s) => Ok(SupportedNetwork::Edo(s)),
            Network::Delphi(s) => Ok(SupportedNetwork::Delphi(s)),
            _ => Err(UnsupportedNetworkError {
                explorer_type: "TzStats".to_owned(),
                network: network.to_string(),
            }),
        }
    }
}

pub struct TzStats {
    network: SupportedNetwork,
}

impl TzStats {
    pub fn new(network: Network) -> Result<Self, UnsupportedNetworkError> {
        Ok(TzStats {
            network: SupportedNetwork::try_from(network)?,
        })
    }

    pub fn endpoint_url(&self) -> &'static str {
        use SupportedNetwork::*;

        match &self.network {
            Main(_) | Beta(_) => "https://tzstats.com",
            Florence(_) => "https://florence.tzstats.com",
            Edo(_) => "https://edo.tzstats.com",
            Delphi(_) => "https://delphi.tzstats.com",
        }
    }

    pub fn api_endpoint_url(&self) -> &'static str {
        use SupportedNetwork::*;

        match &self.network {
            Main(_) | Beta(_) => "https://api.tzstats.com",
            Florence(_) => "https://api.florence.tzstats.com",
            Edo(_) => "https://api.edo.tzstats.com",
            Delphi(_) => "https://api.delphi.tzstats.com",
        }
    }

    pub fn operation_link_prefix(&self) -> &'static str {
        self.endpoint_url()
    }

    pub fn get_bakers(&self) -> Result<Vec<Baker>, Box<dyn Error>> {
        Ok(ureq::get(&format!("{}/explorer/bakers", self.api_endpoint_url()))
            .call()?
            .into_json()?)
    }
}
