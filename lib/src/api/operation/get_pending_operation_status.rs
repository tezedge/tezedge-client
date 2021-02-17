#[derive(Debug)]
pub enum PendingOperationStatus {
    Applied,
    Refused,
    /// transaction is finished but not necessarily successful
    Finished,
}

pub type GetPendingOperationStatusResult = Result<PendingOperationStatus, ()>;

pub trait GetPendingOperationStatus {
    fn get_pending_operation_status(
        &self,
        operation_hash: &str
    ) -> GetPendingOperationStatusResult;
}
