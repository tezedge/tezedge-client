use std::convert::TryInto;
use serde::{Serialize, Serializer};

use crate::crypto::{Prefix, WithPrefix, WithoutPrefix};
use crate::{FromBase58Check, ToBase58Check};
use super::FromPrefixedBase58CheckError;

type PublicKeyInner = [u8; 32];

#[allow(non_camel_case_types)]
#[derive(PartialEq, Debug, Clone)]
pub enum PublicKey {
    edpk(PublicKeyInner),
    sppk(PublicKeyInner),
    p2pk(PublicKeyInner),
}

impl PublicKey {
    pub fn from_base58check(encoded: &str) -> Result<Self, FromPrefixedBase58CheckError> {
        let (prefix, bytes_vec) = encoded
            .from_base58check()?
            .without_any_prefix()?;

        let inner = bytes_vec.try_into()
            .or(Err(FromPrefixedBase58CheckError::InvalidSize))?;

        match prefix {
            Prefix::edpk => Ok(Self::edpk(inner)),
            Prefix::sppk => Ok(Self::sppk(inner)),
            Prefix::p2pk => Ok(Self::p2pk(inner)),
            _ => Err(FromPrefixedBase58CheckError::NotMatchingPrefix)
        }
    }
}

impl ToBase58Check for PublicKey {
    fn to_base58check(&self) -> String {
        self.as_ref()
            .with_prefix(Prefix::edpk)
            .to_base58check()
    }
}

impl AsRef<PublicKeyInner> for PublicKey {
    fn as_ref(&self) -> &PublicKeyInner {
        match self {
            Self::edpk(k) => &k,
            Self::sppk(k) => &k,
            Self::p2pk(k) => &k,
        }
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
