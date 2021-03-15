use crate::NewOperationGroup;

pub type RunOperationResult = Result<serde_json::Value, ()>;

/// Run operation and test the result without signing it first.
pub trait RunOperation {
    fn run_operation(
        &self,
        operation_group: &NewOperationGroup,
    ) -> RunOperationResult;
}
