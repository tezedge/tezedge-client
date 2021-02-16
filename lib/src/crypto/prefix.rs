use failure::Fail;

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

#[derive(Debug, PartialEq, Fail)]
#[fail(display = "not matching prefix")]
pub struct NotMatchingPrefixError;

pub trait WithPrefix {
    type Target;

    /// returns value with prefix prepended.
    fn with_prefix(&self, prefix: Prefix) -> Self::Target;
}

impl WithPrefix for [u8] {
    type Target = Vec<u8>;

    fn with_prefix(&self, prefix: Prefix) -> Self::Target {
        [prefix.as_ref().to_vec(), self.to_vec()].concat()
    }
}

pub trait WithoutPrefix {
    type Target;

    /// returns value with prefix removed.
    fn without_prefix(&self, prefix: Prefix) -> Result<Self::Target, NotMatchingPrefixError>;
}

impl WithoutPrefix for [u8] {
    type Target = Vec<u8>;

    fn without_prefix(&self, prefix: Prefix) -> Result<Self::Target, NotMatchingPrefixError> {
        let prefix_bytes: &[u8] = prefix.as_ref();

        if prefix_bytes.len() > self.len() || prefix_bytes != &self[..prefix_bytes.len()] {
            return Err(NotMatchingPrefixError);
        }

        Ok(self[prefix_bytes.len()..].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matching_prefix_encode_decode() {
        let bytes = vec![Prefix::edsig.as_ref().to_vec(), vec![1, 2, 3, 4]]
            .concat();
        assert_eq!(bytes.without_prefix(Prefix::edsig), Ok(vec![1, 2, 3, 4]));
    }

    #[test]
    fn test_not_matching_prefix_err() {
        let bytes = vec![Prefix::edpk.as_ref().to_vec(), vec![1, 2, 3, 4]]
            .concat();
        assert_eq!(bytes.without_prefix(Prefix::edsig), Err(NotMatchingPrefixError));
    }
}
