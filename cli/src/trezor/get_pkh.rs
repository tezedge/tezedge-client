use console::style;

use lib::PublicKeyHash;
use lib::trezor_api::Trezor;

use crate::common::exit_with_error;
use super::trezor_execute;

pub fn get_pkh(trezor: &mut Trezor, path: Vec<u32>) -> PublicKeyHash {
    let address = trezor_execute(trezor.get_address(path.clone()));
    match PublicKeyHash::from_base58check(&address) {
        Ok(pkh) => pkh,
        Err(_) => {
            exit_with_error(format!(
                "invalid public key hash received from trezor: {}",
                style(&address).red(),
            ));
        }
    }
}
