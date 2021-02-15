use std::convert::TryInto;
use sodiumoxide::{hex, crypto};
use crypto::sign::ed25519;

use crate::crypto::{blake2b, base58check};
use base58check::{ToBase58Check, Prefix};

use super::{SignOperation, SignOperationResult, OperationSignatureInfo};

pub struct LocalSigner {
    pub_key: Vec<u8>,
    priv_key: Vec<u8>,
}

impl LocalSigner {
    pub fn new(pub_key: Vec<u8>, priv_key: Vec<u8>) -> Self {
        Self {
            pub_key,
            priv_key,
        }
    }

    /// concatenated private + public key
    // TODO: refactor
    fn combined_key(&self) -> [u8; 64] {
        vec![self.priv_key.clone(), self.pub_key.clone()]
            .concat()
            .try_into()
            .unwrap()
    }
}

impl SignOperation for LocalSigner {
    fn sign_operation(&self, forged_operation: String) -> SignOperationResult {
        let combined_key = self.combined_key();
        let operation = hex::decode(&forged_operation)?;

        // TODO: add watermarks

        let signature_bytes = ed25519::sign_detached(
            &blake2b::digest_256(&vec![vec![3], operation].concat()),
            &ed25519::SecretKey(combined_key),
        );

        let signature = signature_bytes.as_ref()
            .to_base58check_prefixed(Prefix::edsig);

        let operation_with_signature = format!(
            "{}{}",
            &forged_operation,
            hex::encode(signature_bytes),
        );

        let operation_hash = blake2b::digest_256(
            &hex::decode(&operation_with_signature)?
        ).to_base58check_prefixed(Prefix::operation);


        Ok(OperationSignatureInfo {
            signature,
            operation_with_signature,
            operation_hash,
        })
    }
}
