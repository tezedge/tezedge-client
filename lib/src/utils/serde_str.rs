use std::str::FromStr;
use std::fmt::Display;
use serde::{Serializer, Deserializer, Deserialize};

pub fn serialize<T, S>(value: &T, s: S) -> Result<S::Ok, S::Error>
    where T: ToString,
          S: Serializer,
{
    s.serialize_str(&value.to_string())
}

pub fn deserialize<'de, T, D>(d: D) -> Result<T, D::Error>
    where T: FromStr,
          <T as FromStr>::Err: Display,
          D: Deserializer<'de>,
{
    let val_str: String = Deserialize::deserialize(d)?;
    T::from_str(&val_str)
        .map_err(serde::de::Error::custom)
}
