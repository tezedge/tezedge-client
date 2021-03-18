use crate::{Address, ImplicitAddress};

pub type GetContractManagerAddressResult = Result<ImplicitAddress, ()>;

pub trait GetContractManagerAddress {
    /// Get manager address for given contract.
    ///
    /// - If given address is `ImplicitAddress`, manager address = contract address.
    /// - If given address is `OriginatedAddress`, manager address will
    /// be the one, that originated this contract.
    fn get_contract_manager_address(&self, addr: &Address) -> GetContractManagerAddressResult;
}
