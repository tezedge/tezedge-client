use crate::{
    NewOperationGroup, NewOperation, PublicKey, PublicKeyHash,
    NewDelegationOperation, NewRevealOperation, NewTransactionOperation,
};
use super::{Forge, ForgeNat, Forged};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum OperationTag {
    Endorsement         = 0,
    SeedNonceRevelation = 1,
    DoubleEndorsement   = 2,
    DoubleBaking        = 3,
    Activation          = 4,
    Proposals           = 5,
    Ballot              = 6,
    Reveal              = 107,
    Transaction         = 108,
    Origination         = 109,
    Delegation          = 110,
}

impl ForgeNat for OperationTag {
    fn forge_nat(&self) -> Forged {
        (*self as u32).forge_nat()
    }
}

fn forge_address(pkh: &PublicKeyHash, compact: bool) -> Forged {
    let bytes = match pkh {
        PublicKeyHash::tz1(key) => [vec![0, 0], key.to_vec()].concat(),
        PublicKeyHash::tz2(key) => [vec![0, 1], key.to_vec()].concat(),
        PublicKeyHash::tz3(key) => [vec![0, 2], key.to_vec()].concat(),
        PublicKeyHash::KT1(key) => [vec![1], key.to_vec(), vec![0]].concat(),
    };
    Forged(if compact { bytes[1..].to_vec() } else { bytes })
}

fn forge_delegate_pkh(pkh: &Option<PublicKeyHash>) -> Forged {
    Forged(match pkh.as_ref() {
        Some(pkh) => {
            [
                true.forge().take(),
                forge_address(pkh, true).take(),
            ].concat()
        }
        None => false.forge().take(),
    })
}

impl Forge for PublicKey {
    fn forge(&self) -> Forged {
        Forged(match self {
            PublicKey::edpk(key) => [vec![0], key.to_vec()].concat(),
            PublicKey::sppk(key) => [vec![1], key.to_vec()].concat(),
            PublicKey::p2pk(key) => [vec![2], key.to_vec()].concat(),
        })
    }
}


impl Forge for NewRevealOperation {
    fn forge(&self) -> Forged {
        Forged([
            OperationTag::Reveal.forge_nat().take(),
            forge_address(&self.source, true).take(),
            self.fee.forge_nat().take(),
            self.counter.forge_nat().take(),
            self.gas_limit.forge_nat().take(),
            self.storage_limit.forge_nat().take(),
            self.public_key.forge().take(),
        ].concat())
    }
}

impl Forge for NewTransactionOperation {
    fn forge(&self) -> Forged {
        Forged([
            OperationTag::Transaction.forge_nat().take(),
            forge_address(&self.source, true).take(),
            self.fee.forge_nat().take(),
            self.counter.forge_nat().take(),
            self.gas_limit.forge_nat().take(),
            self.storage_limit.forge_nat().take(),
            self.amount.forge_nat().take(),
            forge_address(&self.destination, false).take(),
            // TODO: replace with forging parameters. At the moment
            // no additional parameters are used, when they will be,
            // this needs to be changed as well.
            false.forge().take(),
        ].concat())
    }
}

impl Forge for NewDelegationOperation {
    fn forge(&self) -> Forged {
        Forged([
            OperationTag::Delegation.forge_nat().take(),
            forge_address(&self.source, true).take(),
            self.fee.forge_nat().take(),
            self.counter.forge_nat().take(),
            self.gas_limit.forge_nat().take(),
            self.storage_limit.forge_nat().take(),
            forge_delegate_pkh(&self.delegate_to).take()
        ].concat())
    }
}

impl Forge for NewOperation {
    fn forge(&self) -> Forged {
        match self {
            NewOperation::Reveal(op) => op.forge(),
            NewOperation::Transaction(op) => op.forge(),
            NewOperation::Delegation(op) => op.forge(),
        }
    }
}

impl Forge for NewOperationGroup {
    fn forge(&self) -> Forged {
        Forged([
            self.branch.forge().take(),
            self.to_operations_vec().into_iter()
                .flat_map(|op| op.forge().take())
                .collect(),
        ].concat())
    }
}
