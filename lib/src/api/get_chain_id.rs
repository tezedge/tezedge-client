use serde::{Serialize, Deserialize};

pub type GetChainIDResult = Result<String, ()>;

pub trait GetChainID {
    fn get_chain_id(&self) -> GetChainIDResult;
}
