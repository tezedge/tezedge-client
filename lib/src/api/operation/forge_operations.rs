use serde::{Serialize, Deserialize};

use super::Operation;

pub type ForgeOperationsResult = Result<String, ()>;

pub trait ForgeOperations {
    fn forge_operations<S>(
        &self,
        last_block_hash: S,
        operations: &[Operation],
    ) -> ForgeOperationsResult
        where S: AsRef<str>;
}
