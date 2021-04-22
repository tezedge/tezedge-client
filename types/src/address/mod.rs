//! Defines address(public key hash) types.

/// Address size in bytes.
pub const ADDRESS_LEN: usize = 20;

mod address;
pub use address::*;

mod implicit_address;
pub use implicit_address::*;

mod originated_address;
pub use originated_address::*;

mod implicit_or_originated_with_manager;
pub use implicit_or_originated_with_manager::*;
