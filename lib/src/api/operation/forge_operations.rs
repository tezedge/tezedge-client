use crate::{BlockHash, NewOperation};

pub type ForgeOperationsResult = Result<String, ()>;

pub trait ForgeOperations {
    fn forge_operations(
        &self,
        last_block_hash: &BlockHash,
        operations: &[NewOperation],
    ) -> ForgeOperationsResult;
}
