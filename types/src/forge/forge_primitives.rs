use crate::BlockHash;
use super::{Forge, ForgeNat, Forged};

impl Forge for bool {
    fn forge(&self) -> Forged {
        Forged(vec![if *self { 255 } else { 0 }])
    }
}

macro_rules! num_forge {
    ($type:ident) => {
        impl Forge for $type {
            fn forge(&self) -> Forged {
                Forged(self.to_be_bytes().to_vec())
            }
        }
    };
}

num_forge!(u32);
num_forge!(u64);

num_forge!(i32);
num_forge!(i64);

macro_rules! num_forge_nat {
    ($type:ident) => {
        impl ForgeNat for $type {
            fn forge_nat(&self) -> Forged {
                let mut num = *self;
                let mut res = vec![(num & 0x7F) as u8];
                num >>= 7;

                while num > 0 {
                    *res.last_mut().unwrap() |= 0x80;
                    res.push((num & 0x7F) as u8);
                    num >>= 7;
                }

                Forged(res)
            }
        }
    };
}

num_forge_nat!(u32);
num_forge_nat!(u64);

impl<T: Forge> Forge for [T] {
    fn forge(&self) -> Forged {
        self.iter()
            .map(|x| x.forge())
            .flatten()
            .collect::<Vec<_>>()
            .forge()
    }
}

impl Forge for [u8] {
    fn forge(&self) -> Forged {
        let mut bytes = (self.len() as u32).forge().take();
        bytes.extend(self);
        Forged(bytes)
    }
}

impl Forge for str {
    fn forge(&self) -> Forged {
        // forge as a byte array
        self.as_bytes().forge()
    }
}

impl Forge for BlockHash {
    fn forge(&self) -> Forged {
        Forged(self.as_ref().to_vec())
    }
}
