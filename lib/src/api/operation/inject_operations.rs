pub type InjectOperationsResult = Result<serde_json::Value, ()>;

pub trait InjectOperations {
    fn inject_operations(
        &self,
        operation_with_signature: &str,
    ) -> InjectOperationsResult;
}
