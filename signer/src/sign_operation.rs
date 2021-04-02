use sodiumoxide::hex;
use trezor_api::protos::TezosSignedTx;

#[derive(Debug, Clone)]
pub struct OperationSignatureInfo {
    /// base58check with prefix(`Prefix::operation`) encoded operation hash.
    pub operation_hash: String,
    /// forged operation(hex) concatenated with signature('hex').
    pub operation_with_signature: String,
    /// operation signature encoded with base58check with prefix (`Prefix::edsig`).
    pub signature: String,
}

impl From<TezosSignedTx> for OperationSignatureInfo {
    fn from(sig_info: TezosSignedTx) -> Self {
        OperationSignatureInfo {
            operation_hash: sig_info.get_operation_hash().to_string(),
            operation_with_signature: hex::encode(
                sig_info.get_sig_op_contents(),
            ),
            signature: sig_info.get_signature().to_string(),
        }
    }
}

pub type SignOperationResult = Result<OperationSignatureInfo, ()>;

pub trait SignOperation {
    // TODO: replace String type with newtype to avoid errors
    fn sign_operation(&self, forged_operation: String) -> SignOperationResult;
}
