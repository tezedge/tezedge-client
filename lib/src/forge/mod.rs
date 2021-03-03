use std::ops::Deref;

mod micheline;
mod forge_primitives;
mod forge_operations;

pub struct Forged(Vec<u8>);

impl Forged {
    pub fn take(self) -> Vec<u8> {
        self.0
    }
}

impl AsRef<[u8]> for Forged {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Deref for Forged {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for Forged {
    type Item = u8;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub trait Forge {
    fn forge(&self) -> Forged;
}

pub trait ForgeNat {
    /// Encode a number using LEB128 encoding (Zarith).
    fn forge_nat(&self) -> Forged;
}
