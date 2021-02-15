use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NodeVersion {
    major: usize,
    minor: usize,
    additional_info: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NetworkVersion {
    chain_name: String,
    distributed_db_version: usize,
    p2p_version: usize,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CommitInfo {
    commit_hash: String,
    commit_date: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct VersionInfo {
    pub node_version: NodeVersion,
    pub network_version: NetworkVersion,
    pub commit_info: CommitInfo,
}

pub type GetVersionInfoResult = Result<VersionInfo, ()>;

pub trait GetVersionInfo {
    fn get_version_info(&self) -> GetVersionInfoResult;
}
