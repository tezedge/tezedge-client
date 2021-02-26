use crate::NewOperationGroup;

pub type PreapplyOperationsResult = Result<serde_json::Value, ()>;

pub trait PreapplyOperations {
    fn preapply_operations(
        &self,
        operation_group: &NewOperationGroup,
        signature: &str,
    ) -> PreapplyOperationsResult;
}
