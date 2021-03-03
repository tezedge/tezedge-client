use std::convert::TryInto;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use trezor_api::protos::{TezosSignTx_TezosContractID, TezosSignTx_TezosContractID_TezosContractType};

use crate::crypto::{Prefix, WithPrefix, WithoutPrefix};
use crate::{FromBase58Check, ToBase58Check};
use super::FromPrefixedBase58CheckError;

type PublicKeyHashInner = [u8; 20];

#[allow(non_camel_case_types)]
#[derive(PartialEq, Debug, Clone)]
pub enum PublicKeyHash {
    tz1(PublicKeyHashInner),
    tz2(PublicKeyHashInner),
    tz3(PublicKeyHashInner),
    KT1(PublicKeyHashInner),
}

impl PublicKeyHash {
    pub fn from_base58check(encoded: &str) -> Result<Self, FromPrefixedBase58CheckError> {
        let (prefix, bytes_vec) = encoded
            .from_base58check()?
            .without_any_prefix()?;

        let inner = bytes_vec.try_into()
            .or(Err(FromPrefixedBase58CheckError::InvalidSize))?;

        match prefix {
            Prefix::tz1 => Ok(Self::tz1(inner)),
            Prefix::tz2 => Ok(Self::tz2(inner)),
            Prefix::tz3 => Ok(Self::tz3(inner)),
            Prefix::KT1 => Ok(Self::KT1(inner)),
            _ => Err(FromPrefixedBase58CheckError::NotMatchingPrefix)
        }
    }

    pub fn get_prefix(&self) -> Prefix {
        match self {
            Self::tz1(_) => Prefix::tz1,
            Self::tz2(_) => Prefix::tz2,
            Self::tz3(_) => Prefix::tz3,
            Self::KT1(_) => Prefix::KT1,
        }
    }
}

impl ToBase58Check for PublicKeyHash {
    fn to_base58check(&self) -> String {
        self.as_ref().with_prefix(self.get_prefix()).to_base58check()
    }
}

impl AsRef<PublicKeyHashInner> for PublicKeyHash {
    fn as_ref(&self) -> &PublicKeyHashInner {
        match self {
            Self::tz1(k) => &k,
            Self::tz2(k) => &k,
            Self::tz3(k) => &k,
            Self::KT1(k) => &k,
        }
    }
}

impl Serialize for PublicKeyHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(
            &self.to_base58check()
        )
    }
}

impl<'de> Deserialize<'de> for PublicKeyHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;

        PublicKeyHash::from_base58check(&encoded)
            .map_err(|err| {
                serde::de::Error::custom(err)
            })
    }
}

impl Into<Vec<u8>> for PublicKeyHash {
    fn into(self) -> Vec<u8> {
        self.as_ref().to_vec()
    }
}

impl Into<TezosSignTx_TezosContractID> for PublicKeyHash {
    fn into(self) -> TezosSignTx_TezosContractID {
        let mut contract_id = TezosSignTx_TezosContractID::new();
        contract_id.set_hash(self.as_ref().to_vec());
        contract_id.set_tag(match &self {
            Self::KT1(_) => TezosSignTx_TezosContractID_TezosContractType::Originated,
            _ => TezosSignTx_TezosContractID_TezosContractType::Implicit,
        });

        contract_id
    }
}
