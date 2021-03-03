use crate::BlockHash;
use super::{Forge, Forged};

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

num_forge!(u8);
num_forge!(u16);
num_forge!(u32);
num_forge!(u64);
num_forge!(usize);

num_forge!(i8);
num_forge!(i16);
num_forge!(i32);
num_forge!(i64);
num_forge!(isize);

impl Forge for BlockHash {
    fn forge(&self) -> Forged {
        Forged(self.as_ref().to_vec())
    }
}
