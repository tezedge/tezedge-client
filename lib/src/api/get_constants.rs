use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Constants {
    pub hard_gas_limit_per_block: String,
    pub hard_storage_limit_per_operation: String,
}

pub type GetConstantsResult = Result<Constants, ()>;

pub trait GetConstants {
    fn get_constants(&self) -> GetConstantsResult;
}
