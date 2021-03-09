use crate::OriginatedAddress;

pub type GetContractStorageResult = Result<serde_json::Value, ()>;

pub trait GetContractStorage {
    fn get_contract_storage(
        &self,
        addr: &OriginatedAddress,
    ) -> GetContractStorageResult;
}
