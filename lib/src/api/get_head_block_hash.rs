use crate::BlockHash;

pub type GetHeadBlockHashResult = Result<BlockHash, ()>;

pub trait GetHeadBlockHash {
    fn get_head_block_hash(&self) -> GetHeadBlockHashResult;
}
