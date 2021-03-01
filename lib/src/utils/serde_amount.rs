use serde::{Serializer, Deserializer, Deserialize};

use super::parse_float_amount;

pub fn serialize<S>(amount: &u64, s: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
{
    s.serialize_str(&amount.to_string())
}

pub fn deserialize<'de, D>(d: D) -> Result<u64, D::Error>
    where D: Deserializer<'de>,
{
    let amount_str: String = Deserialize::deserialize(d)?;
    parse_float_amount(&amount_str)
        .or(Err("invalid_amount".to_string()))
        .map_err(serde::de::Error::custom)
}
