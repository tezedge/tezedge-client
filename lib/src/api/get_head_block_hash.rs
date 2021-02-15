use serde::{Serialize, Deserialize};

pub type GetHeadBlockHashResult = Result<String, ()>;

pub trait GetHeadBlockHash {
    fn get_head_block_hash(&self) -> GetHeadBlockHashResult;
}
