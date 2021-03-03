use super::{ForgeNat, Forged};

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

num_forge_nat!(u8);
num_forge_nat!(u16);
num_forge_nat!(u32);
num_forge_nat!(u64);
num_forge_nat!(usize);
