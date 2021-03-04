use std::convert::TryInto;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use trezor_api::protos::{TezosSignTx_TezosContractID, TezosSignTx_TezosContractID_TezosContractType};

use crate::{ImplicitAddress, OriginatedAddress, crypto::{Prefix, WithPrefix, WithoutPrefix}};
use crate::{FromBase58Check, ToBase58Check};
use super::FromPrefixedBase58CheckError;

#[allow(non_camel_case_types)]
#[derive(PartialEq, Debug, Clone)]
pub enum Address {
    Implicit(ImplicitAddress),
    Originated(OriginatedAddress),
}

impl Address {
    pub fn from_base58check(encoded: &str) -> Result<Self, FromPrefixedBase58CheckError> {
        // TODO: avoid extra base58 decoding here.
        let bytes_vec = encoded.from_base58check()?;

        match Prefix::of(&bytes_vec) {
            Some(Prefix::tz1) |
            Some(Prefix::tz2) |
            Some(Prefix::tz3) => {
                Ok(ImplicitAddress::from_base58check(encoded)?.into())
            }
            Some(Prefix::KT1) => {
                Ok(OriginatedAddress::from_base58check(encoded)?.into())
            }
            _ => Err(FromPrefixedBase58CheckError::NotMatchingPrefix)
        }
    }

    pub fn get_prefix(&self) -> Prefix {
        match self {
            Self::Implicit(addr) => addr.get_prefix(),
            Self::Originated(addr) => addr.get_prefix(),
        }
    }

    pub fn is_implicit(&self) -> bool {
        matches!(self, Self::Implicit(_))
    }

    pub fn is_originated(&self) -> bool {
        matches!(self, Self::Originated(_))
    }

    pub fn as_implicit(self) -> Option<ImplicitAddress> {
        match self {
            Self::Implicit(addr) => Some(addr),
            Self::Originated(_) => None,
        }
    }

    pub fn as_originated(self) -> Option<OriginatedAddress> {
        match self {
            Self::Implicit(_) => None,
            Self::Originated(addr) => Some(addr),
        }
    }
}

impl ToBase58Check for Address {
    fn to_base58check(&self) -> String {
        self.as_ref().with_prefix(self.get_prefix()).to_base58check()
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Implicit(addr) => addr.as_ref(),
            Self::Originated(addr) => addr.as_ref(),
        }
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(
            &self.to_base58check()
        )
    }
}

impl<'de> Deserialize<'de> for Address {
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

impl From<ImplicitAddress> for Address {
    fn from(addr: ImplicitAddress) -> Self {
        Self::Implicit(addr)
    }
}

impl From<OriginatedAddress> for Address {
    fn from(addr: OriginatedAddress) -> Self {
        Self::Originated(addr)
    }
}

impl Into<Vec<u8>> for Address {
    fn into(self) -> Vec<u8> {
        self.as_ref().to_vec()
    }
}

impl Into<TezosSignTx_TezosContractID> for Address {
    fn into(self) -> TezosSignTx_TezosContractID {
        let mut contract_id = TezosSignTx_TezosContractID::new();
        contract_id.set_hash(self.as_ref().to_vec());
        contract_id.set_tag(match &self {
            Self::Implicit(_) => TezosSignTx_TezosContractID_TezosContractType::Implicit,
            Self::Originated(_) => TezosSignTx_TezosContractID_TezosContractType::Originated,
        });

        contract_id
    }
}
