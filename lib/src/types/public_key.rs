use std::convert::TryInto;
use serde::{Serialize, Serializer};

use crate::crypto::{Prefix, WithPrefix, WithoutPrefix};
use crate::{FromBase58Check, ToBase58Check};
use super::FromPrefixedBase58CheckError;

type PublicKeyInner = [u8; 32];

#[derive(Debug, Clone)]
pub struct PublicKey(PublicKeyInner);

impl PublicKey {
    pub fn from_base58check(encoded: &str) -> Result<Self, FromPrefixedBase58CheckError> {
        let key_bytes: PublicKeyInner = encoded
            .from_base58check()?
            .without_prefix(Prefix::edpk)?
            .try_into()
            .or(Err(FromPrefixedBase58CheckError::InvalidSize))?;

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

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(
            &self.to_base58check()
        )
    }
}
