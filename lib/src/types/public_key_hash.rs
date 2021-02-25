use std::convert::TryInto;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use trezor_api::protos::{TezosSignTx_TezosContractID, TezosSignTx_TezosContractID_TezosContractType};

use crate::crypto::{Prefix, WithPrefix, WithoutPrefix};
use crate::{FromBase58Check, ToBase58Check};
use super::FromPrefixedBase58CheckError;

type PublicKeyHashInner = [u8; 20];

#[derive(PartialEq, Debug, Clone)]
pub struct PublicKeyHash(PublicKeyHashInner);

impl PublicKeyHash {
    pub fn from_base58check(encoded: &str) -> Result<Self, FromPrefixedBase58CheckError> {
        let key_bytes: PublicKeyHashInner = encoded
            .from_base58check()?
            .without_prefix(Prefix::tz1)?
            .try_into()
            .or(Err(FromPrefixedBase58CheckError::InvalidSize))?;

        Ok(Self(key_bytes))
    }
}

impl ToBase58Check for PublicKeyHash {
    fn to_base58check(&self) -> String {
        self.0
            .with_prefix(Prefix::tz1)
            .to_base58check()
    }
}

impl AsRef<PublicKeyHashInner> for PublicKeyHash {
    fn as_ref(&self) -> &PublicKeyHashInner {
        &self.0
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
        contract_id.set_tag(TezosSignTx_TezosContractID_TezosContractType::Implicit);

        contract_id
    }
}
