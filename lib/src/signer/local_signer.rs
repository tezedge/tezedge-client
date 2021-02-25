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
}

impl SignOperation for LocalSigner {
    fn sign_operation(&self, forged_operation: String) -> SignOperationResult {
        let combined_key = CombinedKey::new(&self.priv_key, &self.pub_key);
        let operation = hex::decode(&forged_operation)?;

        // TODO: add watermarks

        let signature_bytes = ed25519::sign_detached(
            &blake2b::digest_256(&vec![vec![3], operation].concat()),
            &ed25519::SecretKey(combined_key.as_ref().clone()),
        );

        let signature = signature_bytes.as_ref()
            .with_prefix(Prefix::edsig)
            .to_base58check();

        let operation_with_signature = format!(
            "{}{}",
            &forged_operation,
            hex::encode(signature_bytes),
        );

        let operation_hash = blake2b::digest_256(
            &hex::decode(&operation_with_signature)?
        )
            .with_prefix(Prefix::operation)
            .to_base58check();


        Ok(OperationSignatureInfo {
            signature,
            operation_with_signature,
            operation_hash,
        })
    }
}
