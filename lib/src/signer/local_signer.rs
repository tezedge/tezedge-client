use sodiumoxide::{hex, crypto};
use crypto::sign::ed25519;

use crate::{PublicKey, PrivateKey, CombinedKey, ToBase58Check};
use crate::crypto::{blake2b, Prefix, WithPrefix};

use super::{SignOperation, SignOperationResult, OperationSignatureInfo};

pub struct LocalSigner {
    pub_key: PublicKey,
    priv_key: PrivateKey,
}

impl LocalSigner {
    pub fn new(pub_key: PublicKey, priv_key: PrivateKey) -> Self {
        Self {
            pub_key,
            priv_key,
        }
    }

    // TODO: make separate newtype for ForgedOperation
    pub fn sign_forged_operation_bytes(
        &self,
        operation_bytes: &[u8],
    ) -> OperationSignatureInfo
    {
        let combined_key = CombinedKey::new(&self.priv_key, &self.pub_key);

        let signature_bytes = ed25519::sign_detached(
            &blake2b::digest_256(&[vec![3], operation_bytes.to_vec()].concat()),
            &ed25519::SecretKey(combined_key.as_ref().clone()),
        );

        let signature = signature_bytes.as_ref()
            .with_prefix(Prefix::edsig)
            .to_base58check();

        let operation_with_signature_bytes = [
            operation_bytes.to_vec(),
            signature_bytes.as_ref().to_vec(),
        ].concat();

        let operation_with_signature = hex::encode(&operation_with_signature_bytes);

        let operation_hash = blake2b::digest_256(
            &operation_with_signature_bytes,
        )
            .with_prefix(Prefix::operation)
            .to_base58check();


        OperationSignatureInfo {
            signature,
            operation_with_signature,
            operation_hash,
        }
    }
}

impl SignOperation for LocalSigner {
    fn sign_operation(&self, forged_operation: String) -> SignOperationResult {
        let forged_bytes = hex::decode(&forged_operation)?;
        Ok(self.sign_forged_operation_bytes(&forged_bytes))
    }
}
