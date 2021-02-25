use crate::PublicKeyHash;

pub type GetManagerKeyResult = Result<Option<String>, ()>;

pub trait GetManagerKey {
    /// Get the manager_key hash for the given public key hash.
    fn get_manager_key(&self, pkh: &PublicKeyHash) -> GetManagerKeyResult;
}
