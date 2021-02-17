use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PendingOperation {
    pub branch: String,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PendingOperations {
    pub applied: Vec<PendingOperation>,
    pub refused: Vec<PendingOperation>,
}

pub type GetPendingOperationsResult = Result<PendingOperations, ()>;

pub trait GetPendingOperations {
    fn get_pending_operations(&self) -> GetPendingOperationsResult;
}
