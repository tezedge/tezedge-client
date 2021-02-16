use std::convert::TryInto;

use crate::crypto::{Prefix, WithPrefix, WithoutPrefix};
use crate::crypto::base58check::{FromBase58Check, ToBase58Check};
use super::KeyFromBase58CheckError;

type PublicKeyInner = [u8; 32];

#[derive(Clone)]
pub struct PublicKey(PublicKeyInner);

impl PublicKey {
    pub fn from_base58check(encoded: &str) -> Result<Self, KeyFromBase58CheckError> {
        let key_bytes: PublicKeyInner = encoded
            .from_base58check()?
            .without_prefix(Prefix::edpk)?
            .try_into()
            .or(Err(KeyFromBase58CheckError::InvalidKeySize))?;

        Ok(Self(key_bytes))
    }
}

impl ToBase58Check for PublicKey {
    fn to_base58check(&self) -> String {
        self.0
            .with_prefix(Prefix::edpk)
            .to_base58check()
    }
}

impl AsRef<PublicKeyInner> for PublicKey {
    fn as_ref(&self) -> &PublicKeyInner {
        &self.0
    }
}
