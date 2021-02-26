use crate::NewOperationGroup;

pub type ForgeOperationsResult = Result<String, ()>;

pub trait ForgeOperations {
    fn forge_operations(
        &self,
        operation_group: &NewOperationGroup,
    ) -> ForgeOperationsResult;
}
