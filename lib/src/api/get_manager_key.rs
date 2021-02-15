use serde::{Serialize, Deserialize};

pub type GetManagerKeyResult = Result<Option<String>, ()>;

pub trait GetManagerKey {
    /// Get the manager_key hash for the given key.
    fn get_manager_key<S>(&self, key: S) -> GetManagerKeyResult
        where S: AsRef<str>;
}
