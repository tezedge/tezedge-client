use crate::NewOperation;

pub type ForgeOperationsResult = Result<String, ()>;

pub trait ForgeOperations {
    fn forge_operations<S>(
        &self,
        last_block_hash: S,
        operations: &[NewOperation],
    ) -> ForgeOperationsResult
        where S: AsRef<str>;
}
