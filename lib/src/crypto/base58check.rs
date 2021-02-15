// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use base58::{FromBase58, ToBase58};
use failure::Fail;
use sodiumoxide::crypto::hash::sha256;

#[allow(non_camel_case_types)]
pub enum Prefix {
    tz1,
    tz2,
    tz3,
    KT1,
    B,
    edpk,
    sppk,
    p2pk,
    edsk64,
    edsk32,
    edsig,
    operation,
}

impl AsRef<[u8]> for Prefix {
    fn as_ref(&self) -> &'static [u8] {
        match self {
            Self::tz1 => &[6, 161, 159],
            Self::tz2 => &[6, 161, 161],
            Self::tz3 => &[6, 161, 164],
            Self::KT1 => &[2, 90, 121],
            Self::B => &[1, 52],
            Self::edpk => &[13, 15, 37, 217],
            Self::sppk => &[3, 254, 226, 86],
            Self::p2pk => &[3, 178, 139, 127],
            Self::edsk64 => &[43, 246, 78, 7],
            Self::edsk32 => &[13, 15, 58, 7],
            Self::edsig => &[9, 245, 205, 134, 18],
            Self::operation => &[5, 116],
        }
    }
}

/// Possible errors for base58checked
#[derive(Debug, PartialEq, Fail)]
pub enum FromBase58CheckError {
    /// Base58 error.
    #[fail(display = "invalid base58")]
    InvalidBase58,
    /// The input had invalid checksum.
    #[fail(display = "invalid checksum")]
    InvalidChecksum,
    /// The input is missing checksum.
    #[fail(display = "missing checksum")]
    MissingChecksum,
    /// Provided prefix doesn't match one in base58 string
    #[fail(display = "not matching prefix")]
    NotMatchingPrefix,
}

/// Create double hash of given binary data
fn double_sha256(data: &[u8]) -> sha256::Digest {
    let digest = sha256::hash(data);
    sha256::hash(digest.as_ref())
}

/// A trait for converting a value to base58 encoded string.
pub trait ToBase58Check {
    /// Converts a value of `self` to a base58 value, returning the owned string.
    fn to_base58check(&self) -> String;

    /// Converts a value of `self` to base58 value, prefixing it first with `prefix`.
    fn to_base58check_prefixed(&self, prefix: Prefix) -> String;
}

/// A trait for converting base58check encoded values.
pub trait FromBase58Check {
    /// Size of the checksum used by implementation.
    const CHECKSUM_BYTE_SIZE: usize = 4;

    /// Convert a value of `self`, interpreted as base58check encoded data, into the tuple with version and payload as bytes vector.
    fn from_base58check(&self) -> Result<Vec<u8>, FromBase58CheckError>;

    /// Convert a value of `self`, interpreted as base58check encoded data with prefix, into prefixless payload as bytes vector.
    fn from_base58check_prefixed(&self, prefix: Prefix) -> Result<Vec<u8>, FromBase58CheckError>;
}

impl ToBase58Check for [u8] {
    fn to_base58check(&self) -> String {
        // 4 bytes checksum
        let mut payload = Vec::with_capacity(self.len() + 4);
        payload.extend(self);
        let checksum = double_sha256(self);
        payload.extend(&checksum[..4]);

        payload.to_base58()
    }

    fn to_base58check_prefixed(&self, prefix: Prefix) -> String {
        vec![prefix.as_ref().to_vec(), self.to_vec()]
            .concat()
            .to_base58check()
    }
}

impl FromBase58Check for str {
    fn from_base58check(&self) -> Result<Vec<u8>, FromBase58CheckError> {
        match self.from_base58() {
            Ok(payload) => {
                if payload.len() >= Self::CHECKSUM_BYTE_SIZE {
                    let data_len = payload.len() - Self::CHECKSUM_BYTE_SIZE;
                    let data = &payload[..data_len];
                    let checksum_provided = &payload[data_len..];

                    let checksum_expected = double_sha256(data);
                    let checksum_expected = &checksum_expected[..4];

                    if checksum_expected == checksum_provided {
                        Ok(data.to_vec())
                    } else {
                        Err(FromBase58CheckError::InvalidChecksum)
                    }
                } else {
                    Err(FromBase58CheckError::MissingChecksum)
                }
            }
            Err(_) => Err(FromBase58CheckError::InvalidBase58),
        }
    }

    fn from_base58check_prefixed(&self, prefix: Prefix) -> Result<Vec<u8>, FromBase58CheckError> {
        let decoded = self.from_base58check()?;
        let prefix_bytes: &[u8] = prefix.as_ref();

        if prefix_bytes.len() > self.len() || prefix_bytes != &decoded[..prefix_bytes.len()] {
            return Err(FromBase58CheckError::NotMatchingPrefix);
        }

        // remove prefix and return
        Ok(decoded[prefix_bytes.len()..].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sodiumoxide::hex;

    #[test]
    fn test_encode() {
        let decoded = hex::decode("8eceda2f").unwrap().to_base58check();
        let expected = "QtRAcc9FSRg";
        assert_eq!(expected, &decoded);
    }

    #[test]
    fn test_decode() {
        let decoded = "QtRAcc9FSRg".from_base58check().unwrap();
        let expected = hex::decode("8eceda2f").unwrap();
        assert_eq!(expected, decoded);
    }

    #[test]
    fn test_matching_prefix_encode_decode() {
        let bytes = hex::decode("8eceda2f").unwrap();
        let encoded = bytes.to_base58check_prefixed(Prefix::edsig);
        let decode_result = encoded.from_base58check_prefixed(Prefix::edsig);
        assert_eq!(decode_result, Ok(bytes));
    }

    #[test]
    fn test_not_matching_prefix_err() {
        let encoded = hex::decode("8eceda2f").unwrap()
            .to_base58check_prefixed(Prefix::edpk);
        let decode_result = encoded.from_base58check_prefixed(Prefix::edsig);
        assert_eq!(decode_result, Err(FromBase58CheckError::NotMatchingPrefix));
    }
}
