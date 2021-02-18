pub type GetCounterForKeyResult = Result<u64, ()>;

pub trait GetCounterForKey {
    fn get_counter_for_key<S>(&self, key: S) -> GetCounterForKeyResult
        where S: AsRef<str>;
}
