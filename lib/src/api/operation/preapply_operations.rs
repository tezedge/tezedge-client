// use serde::{Serialize, Deserialize};

use crate::signer::OperationSignatureInfo;
use super::Operation;

pub type PreapplyOperationsResult = Result<serde_json::Value, ()>;

pub trait PreapplyOperations {
    fn preapply_operations(
        &self,
        next_protocol_hash: &str,
        last_block_hash: &str,
        signature: &str,
        operations: &[Operation],
    ) -> PreapplyOperationsResult;
}
