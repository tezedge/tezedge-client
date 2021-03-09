use crate::{Address, ImplicitAddress};

pub type GetManagerAddressResult = Result<ImplicitAddress, ()>;

pub trait GetManagerAddress {
    /// Get manager address for given address.
    ///
    /// If given address is `ImplicitAddress`, manager address will be the same.
    ///
    /// If given address is `OriginatedAddress`, manager address will
    /// be the one, that originated this contract.
    fn get_manager_address(&self, addr: &Address) -> GetManagerAddressResult;
}
