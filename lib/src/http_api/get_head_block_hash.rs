use crate::api::{
    GetHeadBlockHash, GetHeadBlockHashResult,
    TransportError, GetHeadBlockHashError,
};
use super::HttpApi;

fn get_head_block_hash_url(base_url: &str) -> String {
    format!("{}/chains/main/blocks/head/hash", base_url)
}

impl From<ureq::Error> for GetHeadBlockHashError {
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

impl From<std::io::Error> for GetHeadBlockHashError {
    fn from(error: std::io::Error) -> Self {
        Self::Transport(TransportError(Box::new(error)))
    }
}

impl GetHeadBlockHash for HttpApi {
    fn get_head_block_hash(&self) -> GetHeadBlockHashResult {
        Ok(self.client.get(&get_head_block_hash_url(&self.base_url))
            .call()?
            .into_json()?)
    }
}
