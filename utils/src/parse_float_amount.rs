use std::str::FromStr;
use rust_decimal::{Decimal, prelude::ToPrimitive};

// TODO: better error handling
/// Convert string amount notation into number in milions
///
/// # Examples:
/// ```rust
/// # use lib::utils::{parse_float_amount};
///
/// assert_eq!(parse_float_amount("1"), Ok("1000000".to_string()));
/// assert_eq!(parse_float_amount("1.35"), Ok("1350000".to_string()));
/// assert_eq!(parse_float_amount("1.05"), Ok("1050000".to_string()));
/// assert_eq!(parse_float_amount(".13"), Ok("130000".to_string()));
/// assert_eq!(parse_float_amount("0.000005"), Ok("5".to_string()));
/// assert_eq!(parse_float_amount("1.12.0"), Err(()));
/// assert_eq!(parse_float_amount("1.1a2"), Err(()));
/// ```
pub fn parse_float_amount(amount: &str) -> Result<u64, ()> {
    let num = Decimal::from_str(amount)
        .or(Err(()))?
        .abs();

    (num * Decimal::from(1_000_000))
        .trunc()
        .to_u64()
        .ok_or(())
}
