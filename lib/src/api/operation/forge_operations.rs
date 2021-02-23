use crate::{BlockHash, NewOperationGroup};

pub type ForgeOperationsResult = Result<String, ()>;

pub trait ForgeOperations {
    fn forge_operations(
        &self,
        last_block_hash: &BlockHash,
        operation_group: &NewOperationGroup,
    ) -> ForgeOperationsResult;
}
