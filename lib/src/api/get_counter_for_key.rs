use crate::Address;

pub type GetCounterForKeyResult = Result<u64, ()>;

pub trait GetCounterForKey {
    fn get_counter_for_key(&self, key: &Address) -> GetCounterForKeyResult;
}
