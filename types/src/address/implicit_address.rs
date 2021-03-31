use std::convert::TryInto;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use trezor_api::protos::{TezosSignTx_TezosContractID, TezosSignTx_TezosContractID_TezosContractType};

use crypto::{Prefix, WithPrefix, WithoutPrefix};
use crypto::base58check::{FromBase58Check, ToBase58Check};
use crate::{Forge, FromPrefixedBase58CheckError};
use super::ADDRESS_LEN;

type ImplicitAddressInner = [u8; ADDRESS_LEN];

#[allow(non_camel_case_types)]
#[derive(PartialEq, Debug, Clone)]
pub enum ImplicitAddress {
    tz1(ImplicitAddressInner),
    tz2(ImplicitAddressInner),
    tz3(ImplicitAddressInner),
}

impl ImplicitAddress {
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
            _ => Err(FromPrefixedBase58CheckError::NotMatchingPrefix)
        }
    }

    pub fn get_prefix(&self) -> Prefix {
        match self {
            Self::tz1(_) => Prefix::tz1,
            Self::tz2(_) => Prefix::tz2,
            Self::tz3(_) => Prefix::tz3,
        }
    }
}

impl ToBase58Check for ImplicitAddress {
    fn to_base58check(&self) -> String {
        self.as_ref().with_prefix(self.get_prefix()).to_base58check()
    }
}

impl AsRef<ImplicitAddressInner> for ImplicitAddress {
    fn as_ref(&self) -> &ImplicitAddressInner {
        match self {
            Self::tz1(k) => &k,
            Self::tz2(k) => &k,
            Self::tz3(k) => &k,
        }
    }
}

impl Serialize for ImplicitAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(
            &self.to_base58check()
        )
    }
}

impl<'de> Deserialize<'de> for ImplicitAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;

        Self::from_base58check(&encoded)
            .map_err(|err| {
                serde::de::Error::custom(err)
            })
    }
}

impl Into<TezosSignTx_TezosContractID> for ImplicitAddress {
    fn into(self) -> TezosSignTx_TezosContractID {
        let mut contract_id = TezosSignTx_TezosContractID::new();
        contract_id.set_hash(self.forge().take());
        contract_id.set_tag(TezosSignTx_TezosContractID_TezosContractType::Implicit);

        contract_id
    }
}
