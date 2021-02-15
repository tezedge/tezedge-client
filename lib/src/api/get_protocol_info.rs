use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ProtocolInfo {
    pub protocol_hash: String,
    pub next_protocol_hash: String,
}

pub type GetProtocolInfoResult = Result<ProtocolInfo, ()>;

pub trait GetProtocolInfo {
    fn get_protocol_info(&self) -> GetProtocolInfoResult;
}
