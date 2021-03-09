use serde::Deserialize;

use crate::{ImplicitAddress, OriginatedAddressWithManager};

/// Either `ImplicitAddress` or `OriginatedAddress` contianing
/// manager's `ImplicitAddress`.
///
/// To create transaction, delegation operation for originated accounts
/// we need to know manager's address, which originated this account.
#[derive(PartialEq, Debug, Clone)]
pub enum ImplicitOrOriginatedWithManager {
    Implicit(ImplicitAddress),
    OriginatedWithManager(OriginatedAddressWithManager),
}

impl From<ImplicitAddress> for ImplicitOrOriginatedWithManager {
    fn from(addr: ImplicitAddress) -> Self {
        Self::Implicit(addr)
    }
}

impl From<OriginatedAddressWithManager> for ImplicitOrOriginatedWithManager {
    fn from(addr_with_manager: OriginatedAddressWithManager) -> Self {
        Self::OriginatedWithManager(addr_with_manager)
    }
}
