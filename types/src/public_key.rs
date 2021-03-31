use std::convert::TryInto;
use serde::{Serialize, Serializer};

use crypto::{blake2b, Prefix, WithPrefix, WithoutPrefix};
use crypto::base58check::{FromBase58Check, ToBase58Check};
use crate::ImplicitAddress;
use super::FromPrefixedBase58CheckError;

pub const PUBLIC_KEY_LEN: usize = 32;
type PublicKeyInner = [u8; PUBLIC_KEY_LEN];

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

    fn curve_prefix(&self) -> u8 {
        match self {
            Self::edpk(_) => 0x01,
            Self::sppk(_) => 0x02,
            Self::p2pk(_) => 0x03,
        }
    }

    /// Hash of the public key, gives it's corresponding address.
    pub fn hash(&self) -> ImplicitAddress {
        match self {
            Self::edpk(key)
                | Self::sppk(key)
                | Self::p2pk(key)
            => {
                let hash = blake2b::digest_160(key);
                let addr_bytes: [u8; 20] = hash
                    .try_into()
                    // unwrap can't fail since blake2b outputs 160 bit digest,
                    // which is exactly 20 bytes, same as `addr_bytes` size.
                    .unwrap();

                match self {
                    Self::edpk(_) => ImplicitAddress::tz1(addr_bytes),
                    Self::sppk(_) => ImplicitAddress::tz2(addr_bytes),
                    Self::p2pk(_) => ImplicitAddress::tz3(addr_bytes),
                }
            }
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
