use std::convert::TryInto;

use crate::crypto::{Prefix, WithPrefix, WithoutPrefix};
use crate::crypto::base58check::{FromBase58Check, ToBase58Check};
use super::KeyFromBase58CheckError;

type PrivateKeyInner = [u8; 32];

#[derive(Clone)]
pub struct PrivateKey(PrivateKeyInner);

impl PrivateKey {
    pub fn from_base58check(encoded: &str) -> Result<Self, KeyFromBase58CheckError> {
        let key_bytes: PrivateKeyInner = encoded
            .from_base58check()?
            .without_prefix(Prefix::edsk32)?
            .try_into()
            .or(Err(KeyFromBase58CheckError::InvalidKeySize))?;

        Ok(Self(key_bytes))
    }
}

impl ToBase58Check for PrivateKey {
    fn to_base58check(&self) -> String {
        self.0
            .with_prefix(Prefix::edsk32)
            .to_base58check()
    }
}

impl AsRef<PrivateKeyInner> for PrivateKey {
    fn as_ref(&self) -> &PrivateKeyInner {
        &self.0
    }
}
