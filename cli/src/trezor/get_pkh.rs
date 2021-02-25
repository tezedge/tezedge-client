use lib::PublicKeyHash;
use lib::trezor_api::Trezor;

use super::trezor_execute;

pub fn get_pkh(trezor: &mut Trezor, path: Vec<u32>) -> PublicKeyHash {
    let address = trezor_execute(trezor.get_address(path));
    PublicKeyHash::from_base58check(&address).unwrap()
}
