use crate::Address;

pub type GetManagerKeyResult = Result<Option<String>, ()>;

pub trait GetManagerKey {
    /// Get the manager_key hash for the given public key hash.
    fn get_manager_key(&self, addr: &Address) -> GetManagerKeyResult;
}
