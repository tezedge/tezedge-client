/// Defines traits with their types, errors, of the available
/// api actions/operations.

pub mod operation;
pub use operation::*;

pub mod contract;
pub use contract::*;

pub mod get_version_info;
pub use get_version_info::*;

pub mod get_constants;
pub use get_constants::*;

pub mod get_protocol_info;
pub use get_protocol_info::*;

pub mod get_head_block_hash;
pub use get_head_block_hash::*;

pub mod get_chain_id;
pub use get_chain_id::*;

// TODO: move inside contract/ and rename to get_contract_counter
pub mod get_counter_for_key;
pub use get_counter_for_key::*;

// TODO: move inside contract/ and rename
pub mod get_manager_key;
pub use get_manager_key::*;

// TODO: move inside contract/ and rename
pub mod get_manager_address;
pub use get_manager_address::*;
