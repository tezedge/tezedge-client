use crate::Address;

pub type GetManagerKeyResult = Result<Option<String>, ()>;

pub trait GetManagerKey {
    /// Get public key for given address.
    ///
    /// If account is not yet revealed, it will return `Ok(None)`.
    fn get_manager_key(&self, addr: &Address) -> GetManagerKeyResult;
}
