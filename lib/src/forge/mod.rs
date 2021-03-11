use std::ops::Deref;

mod prim_type;
pub mod micheline;
mod forge_primitives;
mod forge_transaction_parameters;
mod forge_operations;

#[derive(PartialEq, Debug, Clone)]
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

impl<'a> IntoIterator for &'a Forged {
    type Item = &'a u8;
    type IntoIter = std::slice::Iter<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Forge for Forged {
    /// Simply clones already forged value.
    fn forge(&self) -> Forged {
        self.clone()
    }
}

pub trait Forge {
    fn forge(&self) -> Forged;
}

pub trait ForgeNat {
    /// Encode a number using LEB128 encoding (Zarith).
    fn forge_nat(&self) -> Forged;
}
