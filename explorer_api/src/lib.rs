mod tzstats;
pub use tzstats::*;

#[derive(thiserror::Error, Debug)]
#[error("{explorer_type} explorer doesn't support network version: {network}.")]
pub struct UnsupportedNetworkError {
    pub explorer_type: String,
    pub network: String,
}

pub enum Explorer {
    TzStats(TzStats),
}

impl Explorer {
    pub fn endpoint_url(&self) -> String {
        match self {
            Self::TzStats(explorer) => explorer.endpoint_url().to_owned(),
        }
    }

    pub fn api_endpoint_url(&self) -> String {
        match self {
            Self::TzStats(explorer) => explorer.api_endpoint_url().to_owned(),
        }
    }

    pub fn operation_link_prefix(&self) -> String {
        match self {
            Self::TzStats(explorer) => explorer.operation_link_prefix().to_owned(),
        }
    }
}
