use serde::Deserialize;
use serde_json::Value as SerdeValue;

use crate::{Address, BlockHash, ImplicitAddress, NewOperationGroup, NewOperationWithKind, OriginatedAddress, ToBase58Check};
use crate::api::{
    GetVersionInfo, GetVersionInfoResult, VersionInfo, NodeVersion, NetworkVersion, CommitInfo,
    GetConstants, GetConstantsResult,
    GetProtocolInfo, GetProtocolInfoResult, ProtocolInfo,
    GetHeadBlockHash, GetHeadBlockHashResult,
    GetChainID, GetChainIDResult,
    GetContractStorage, GetContractStorageResult,
    GetCounterForKey, GetCounterForKeyResult,
    GetManagerKey, GetManagerKeyResult,
    GetManagerAddress, GetManagerAddressResult,
    GetPendingOperations, GetPendingOperationsResult, PendingOperation,
    GetPendingOperationStatus, GetPendingOperationStatusResult, PendingOperationStatus,
    ForgeOperations, ForgeOperationsResult,
    PreapplyOperations, PreapplyOperationsResult,
    InjectOperations, InjectOperationsResult,
};

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

    fn get_protocol_info_url(&self) -> String {
        format!(
            "{}/chains/main/blocks/head/protocols",
            self.base_url,
        )
    }

    fn get_head_block_hash_url(&self) -> String {
        format!(
            "{}/chains/main/blocks/head/hash",
            self.base_url,
        )
    }

    fn get_chain_id_url(&self) -> String {
        format!(
            "{}/chains/main/chain_id",
            self.base_url,
        )
    }

    fn get_counter_for_key_url(&self, key: &Address) -> String {
        format!(
            "{}/chains/main/blocks/head/context/contracts/{}/counter",
            self.base_url,
            key.to_base58check(),
        )
    }

    fn get_contract_storage_url(&self, addr: &OriginatedAddress) -> String {
        format!(
            "{}/chains/main/blocks/head/context/contracts/{}/storage",
            self.base_url,
            addr.to_base58check(),
        )
    }

    /// Get manager key
    fn get_manager_key_url(&self, addr: &Address) -> String {
        format!(
            "{}/chains/main/blocks/head/context/contracts/{}/manager_key",
            self.base_url,
            addr.to_base58check(),
        )
    }

    fn get_pending_operations_url(&self) -> String {
        format!(
            "{}/chains/main/mempool/pending_operations",
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

#[derive(Deserialize)]
struct ProtocolInfoJson {
    protocol: String,
    next_protocol: String,
}

impl Into<ProtocolInfo> for ProtocolInfoJson {
    fn into(self) -> ProtocolInfo {
        let mut info = ProtocolInfo::default();
        info.protocol_hash = self.protocol;
        info.next_protocol_hash = self.next_protocol;
        info
    }
}

impl GetProtocolInfo for HttpApi {
    fn get_protocol_info(&self) -> GetProtocolInfoResult {
        Ok(self.client.get(&self.get_protocol_info_url())
            .call()
            .unwrap()
            .into_json::<ProtocolInfoJson>()
            .unwrap()
            .into())
    }
}

impl GetHeadBlockHash for HttpApi {
    fn get_head_block_hash(&self) -> GetHeadBlockHashResult {
        Ok(self.client.get(&self.get_head_block_hash_url())
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

impl GetContractStorage for HttpApi {
    fn get_contract_storage(
        &self,
        addr: &OriginatedAddress,
    ) -> GetContractStorageResult
    {
        Ok(self.client.get(&self.get_contract_storage_url(addr))
           .call()
           .or(Err(()))?
           .into_json()
           .or(Err(()))?)
    }
}

impl GetCounterForKey for HttpApi {
    fn get_counter_for_key(&self, key: &Address) -> GetCounterForKeyResult {
        Ok(self.client.get(&self.get_counter_for_key_url(key))
           .call()
           .unwrap()
           .into_json::<String>()
           .unwrap()
           .parse()
           .unwrap())
    }
}

// TODO: receiving NULL, probably because node isn't synced
impl GetManagerKey for HttpApi {
    fn get_manager_key(&self, addr: &Address) -> GetManagerKeyResult {
        Ok(self.client.get(&self.get_manager_key_url(addr))
           .call()
           .unwrap()
           .into_json::<Option<String>>()
           .unwrap())
    }
}

impl GetManagerAddress for HttpApi {
    fn get_manager_address(&self, addr: &Address) -> GetManagerAddressResult {
        Ok(match addr {
            Address::Implicit(addr) => addr.clone(),
            Address::Originated(addr) => {
                let storage = self.get_contract_storage(addr)?;
                let manager_str = storage["string"].as_str().ok_or(())?;
                ImplicitAddress::from_base58check(manager_str).or(Err(()))?
            }
        })
    }
}

impl GetPendingOperations for HttpApi {
    fn get_pending_operations(&self) -> GetPendingOperationsResult {
        Ok(self.client.get(&self.get_pending_operations_url())
           .call()
           .unwrap()
           .into_json()
           .unwrap())
    }
}

impl GetPendingOperationStatus for HttpApi {
    fn get_pending_operation_status(
        &self,
        operation_hash: &str
    ) -> GetPendingOperationStatusResult
    {
        let pending_operations = self.get_pending_operations()?;

        let contained_by = |ops: &[PendingOperation]| {
            ops.iter()
                .find(|op| op.hash == operation_hash)
                .is_some()
        };

        let status = if contained_by(&pending_operations.applied) {
            PendingOperationStatus::Applied
        } else if contained_by(&pending_operations.refused) {
            PendingOperationStatus::Refused
        } else {
            PendingOperationStatus::Finished
        };

        Ok(status)
    }
}

impl ForgeOperations for HttpApi {
    fn forge_operations(
        &self,
        operation_group: &NewOperationGroup,
    ) -> ForgeOperationsResult
    {
        Ok(self.client.post(&self.forge_operations_url(&operation_group.branch))
           .send_json(dbg!(ureq::json!({
               "branch": &operation_group.branch,
               "contents": operation_group.to_operations_vec()
                   .into_iter()
                   .map(|op| NewOperationWithKind::from(op))
                   .collect::<Vec<_>>(),
           })))
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
