use crate::Address;

pub type GetContractCounterResult = Result<u64, ()>;

pub trait GetContractCounter {
    /// Get counter for a contract.
    fn get_contract_counter(&self, address: &Address) -> GetContractCounterResult;
}
