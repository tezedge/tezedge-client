use std::convert::TryInto;

use super::{PrivateKey, PublicKey};

type CombinedKeyInner = [u8; 64];

/// `PrivateKey` concatenated with `PublicKey`.
#[derive(Clone)]
pub struct CombinedKey(CombinedKeyInner);

impl CombinedKey {
    /// Build `CombinedKey` from private and public key pair.
    ///
    /// Used for signing operations.
    ///
    /// # Examples:
    /// ```rust
    /// # use lib::{PublicKey, PrivateKey, CombinedKey};
    ///
    /// let pub_key = PublicKey::from_base58check("edpkvDFBqnw7WyvKjQMf1WcCnbeocqMwASys3Te4Z9gaznyfzuPFiU").unwrap();
    /// let priv_key = PrivateKey::from_base58check("edsk3NmghEMdi8CFKU3VwJKfzmGbPvBTVukhEXqe4XuXKRYbvx4mxo").unwrap();
    ///
    /// let combined_key = CombinedKey::new(&priv_key, &pub_key);
    /// ```
    pub fn new(priv_key: &PrivateKey, pub_key: &PublicKey) -> Self {
        let inner: CombinedKeyInner = vec![
                priv_key.as_ref().clone(),
                pub_key.as_ref().clone(),
            ]
            .concat()
            .try_into()
            // unwrap here is fine since it can only panic if sizeof
            // pub_key + priv_key is not equal to CombinedKeyInner array size.
            // If thats the case above doc test will fail.
            .unwrap();

        CombinedKey(inner)
    }
}

impl From<(&PrivateKey, &PublicKey)> for CombinedKey {
    fn from((priv_key, pub_key): (&PrivateKey, &PublicKey)) -> Self {
        Self::new(priv_key, pub_key)
    }
}

impl AsRef<CombinedKeyInner> for CombinedKey {
    fn as_ref(&self) -> &CombinedKeyInner {
        &self.0
    }
}
