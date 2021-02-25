// use serde::{Serialize, Deserialize};

use crate::{BlockHash, NewOperationGroup};

pub type PreapplyOperationsResult = Result<serde_json::Value, ()>;

pub trait PreapplyOperations {
    fn preapply_operations(
        &self,
        next_protocol_hash: &str,
        last_block_hash: &BlockHash,
        signature: &str,
        operation_group: &NewOperationGroup,
    ) -> PreapplyOperationsResult;
}
