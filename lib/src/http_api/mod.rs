use serde::Deserialize;
use serde_json::Value as SerdeValue;

use crate::{BlockHash, NewOperationGroup, NewOperationWithKind, ToBase58Check};
use crate::api::{
    GetVersionInfo, GetVersionInfoResult, VersionInfo, NodeVersion, NetworkVersion, CommitInfo,
    GetConstants, GetConstantsResult,
    GetChainID, GetChainIDResult,
    ForgeOperations, ForgeOperationsResult,
    RunOperation, RunOperationResult,
    PreapplyOperations, PreapplyOperationsResult,
    InjectOperations, InjectOperationsResult,
};

mod contract;
pub use contract::*;

mod operation;
pub use operation::*;

mod get_protocol_info;
pub use get_protocol_info::*;

mod get_head_block_hash;
pub use get_head_block_hash::*;

mod get_manager_public_key;
pub use get_manager_public_key::*;

pub struct HttpApi {
    base_url: String,
    client: ureq::Agent,
}

impl HttpApi {
    pub fn new<S: AsRef<str>>(base_url: S) -> Self {
        Self {
            base_url: base_url.as_ref().to_owned(),
            client: ureq::agent(),
        }
    }

    fn get_version_info(&self) -> String {
        format!("{}/version", self.base_url)
    }

    fn get_constants_url(&self) -> String {
        format!(
            "{}/chains/main/blocks/head/context/constants",
            self.base_url,
        )
    }

    fn get_chain_id_url(&self) -> String {
        format!(
            "{}/chains/main/chain_id",
            self.base_url,
        )
    }

    // TODO: add /monitor/bootstrapped  endpoint

    fn forge_operations_url(&self, branch: &BlockHash) -> String {
        format!(
            "{}/chains/main/blocks/{}/helpers/forge/operations",
            self.base_url,
            branch.to_base58check(),
        )
    }

    fn run_operation_url(&self) -> String {
        format!(
            "{}/chains/main/blocks/head/helpers/scripts/run_operation",
            self.base_url,
        )
    }

    fn preapply_operations_url(&self) -> String {
        format!(
            "{}/chains/main/blocks/head/helpers/preapply/operations",
            self.base_url,
        )
    }

    fn inject_operations_url(&self) -> String {
        format!(
            "{}/injection/operation",
            self.base_url,
        )
    }
}

#[derive(Deserialize)]
struct VersionInfoJson {
    version: NodeVersion,
    network_version: NetworkVersion,
    commit_info: CommitInfo
}

impl Into<VersionInfo> for VersionInfoJson {
    fn into(self) -> VersionInfo {
        let mut info = VersionInfo::default();
        info.node_version = self.version;
        info.network_version = self.network_version;
        info.commit_info = self.commit_info;
        info
    }
}

impl GetVersionInfo for HttpApi {
    fn get_version_info(&self) -> GetVersionInfoResult {
        Ok(self.client.post(&self.get_version_info())
            .call()
            .unwrap()
            .into_json::<VersionInfoJson>()
            .unwrap()
            .into())
    }
}

impl GetConstants for HttpApi {
    fn get_constants(&self) -> GetConstantsResult {
        Ok(self.client.get(&self.get_constants_url())
            .call()
            .unwrap()
            .into_json()
            .unwrap())
    }
}

impl GetChainID for HttpApi {
    fn get_chain_id(&self) -> GetChainIDResult {
        Ok(self.client.get(&self.get_chain_id_url())
            .call()
            .unwrap()
            .into_json()
            .unwrap())
    }
}

impl ForgeOperations for HttpApi {
    fn forge_operations(
        &self,
        operation_group: &NewOperationGroup,
    ) -> ForgeOperationsResult
    {
        Ok(self.client.post(&self.forge_operations_url(&operation_group.branch))
           .send_json(ureq::json!({
               "branch": &operation_group.branch,
               "contents": operation_group.to_operations_vec()
                   .into_iter()
                   .map(|op| NewOperationWithKind::from(op))
                   .collect::<Vec<_>>(),
           }))
           .unwrap()
           .into_json()
           .unwrap())
    }
}

impl RunOperation for HttpApi {
    fn run_operation(
        &self,
        operation_group: &NewOperationGroup,
    ) -> RunOperationResult
    {
        Ok(self.client.post(&self.run_operation_url())
           .send_json(ureq::json!({
                "chain_id": self.get_chain_id()?,
                "operation": {
                    "branch": &operation_group.branch,
                    // this is necessary to be valid signature but doesn't
                    // need to match the actual operation signature.
                    "signature": "edsigthZLBZKMBUCwHpMCXHkGtBSzwh7wdUxqs7C1LRMk64xpcVU8tyBDnuFuf9CLkdL3urGem1zkHXFV9JbBBabi6k8QnhW4RG",
                    "contents": operation_group.to_operations_vec()
                        .into_iter()
                        .map(|op| NewOperationWithKind::from(op))
                        .collect::<Vec<_>>(),
                },

           }))
           .unwrap()
           .into_json()
           .unwrap())
    }
}

impl PreapplyOperations for HttpApi {
    fn preapply_operations(
        &self,
        operation_group: &NewOperationGroup,
        signature: &str,
    ) -> PreapplyOperationsResult
    {
        Ok(self.client.post(&self.preapply_operations_url())
           .send_json(ureq::json!([{
               "protocol": &operation_group.next_protocol_hash,
               "branch": &operation_group.branch,
               "signature": signature,
               "contents": operation_group.to_operations_vec()
                   .into_iter()
                   .map(|op| NewOperationWithKind::from(op))
                   .collect::<Vec<_>>(),
           }]))
           .unwrap()
           .into_json()
           .unwrap())
    }
}

impl InjectOperations for HttpApi {
    fn inject_operations(
        &self,
        operation_with_signature: &str,
    ) -> InjectOperationsResult
    {
        let operation_with_signature_json =
            SerdeValue::String(operation_with_signature.to_owned());

        Ok(self.client.post(&self.inject_operations_url())
           .send_json(operation_with_signature_json)
           .unwrap()
           .into_json()
           .unwrap())
    }
}
