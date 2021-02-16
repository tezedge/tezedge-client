use sodiumoxide::crypto;
use crypto::sign::ed25519;

pub use ed25519::Signature;

#[derive(Debug, Clone)]
pub struct OperationSignatureInfo {
    /// base58check with prefix(`Prefix::operation`) encoded operation hash.
    pub operation_hash: String,
    /// forged operation(hex) concatenated with signature('hex').
    pub operation_with_signature: String,
    /// operation signature encoded with base58check with prefix (`Prefix::edsig`).
    pub signature: String,
}

pub type SignOperationResult = Result<OperationSignatureInfo, ()>;

pub trait SignOperation {
    // TODO: replace String type with newtype to avoid errors
    fn sign_operation(&self, forged_operation: String) -> SignOperationResult;
}
