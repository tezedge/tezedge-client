use console::style;

use lib::{ImplicitAddress, KeyDerivationPath};
use lib::trezor_api::Trezor;

use crate::common::exit_with_error;
use super::trezor_execute;

pub fn get_address(trezor: &mut Trezor, path: &KeyDerivationPath) -> ImplicitAddress {
    let address = trezor_execute(trezor.get_address(path));
    match ImplicitAddress::from_base58check(&address) {
        Ok(addr) => addr,
        Err(_) => {
            exit_with_error(format!(
                "invalid public key hash received from trezor: {}",
                style(&address).red(),
            ));
        }
    }
}
