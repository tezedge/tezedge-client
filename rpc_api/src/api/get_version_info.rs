use std::fmt::{self, Display};
use serde::{Serialize, Deserialize};

use crate::api::TransportError;

const MAINNET_CHAINS: [&'static str; 2] = ["TEZOS_BETANET", "TEZOS_MAINNET"];

#[derive(thiserror::Error, Debug)]
pub enum GetVersionInfoError {
    Transport(#[from] TransportError),
    Unknown(String),
}

impl Display for GetVersionInfoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "getting version information failed! Reason: ")?;
        match self {
            Self::Transport(err) => err.fmt(f),
            Self::Unknown(err) => write!(f, "Unknown! {}", err)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct NodeVersion {
    major: usize,
    minor: usize,
    additional_info: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct NetworkVersion {
    pub chain_name: String,
    pub distributed_db_version: usize,
    pub p2p_version: usize,
}

impl NetworkVersion {
    pub fn is_mainnet(&self) -> bool {
        MAINNET_CHAINS.iter()
            .any(|chain| self.chain_name.starts_with(chain))
    }

    /// Explorer url for a given network.
    pub fn explorer_url(&self) -> Option<String> {
        if self.is_mainnet() {
            Some("https://tzstats.com".to_owned())
        } else if self.chain_name.starts_with("TEZOS_DELPHI") {
            Some("https://delphi.tzstats.com".to_owned())
        } else if self.chain_name.starts_with("TEZOS_EDO") {
            Some("https://edo.tzstats.com".to_owned())
        } else if self.chain_name.starts_with("TEZOS_FLORENCE") {
            Some("https://florence.tzstats.com".to_owned())
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct CommitInfo {
    pub commit_hash: String,
    pub commit_date: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct VersionInfo {
    #[serde(rename = "version")]
    pub node_version: NodeVersion,
    pub network_version: NetworkVersion,
    pub commit_info: CommitInfo,
}

impl VersionInfo {
    pub fn is_mainnet(&self) -> bool {
        self.network_version.is_mainnet()
    }

    /// Explorer url for a given network.
    pub fn explorer_url(&self) -> Option<String> {
        self.network_version.explorer_url()
    }
}

pub type GetVersionInfoResult = Result<VersionInfo, GetVersionInfoError>;

pub trait GetVersionInfo {
    fn get_version_info(&self) -> GetVersionInfoResult;
}
