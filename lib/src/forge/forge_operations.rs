use crate::{
    Address, ImplicitAddress, OriginatedAddress,
    NewOperationGroup, NewOperation, PublicKey,
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

impl Forge for ImplicitAddress {
    /// Forge an implicit(tz1, tz2, tz3) address.
    fn forge(&self) -> Forged {
        Forged(match self {
            ImplicitAddress::tz1(key) => [vec![0], key.to_vec()].concat(),
            ImplicitAddress::tz2(key) => [vec![1], key.to_vec()].concat(),
            ImplicitAddress::tz3(key) => [vec![2], key.to_vec()].concat(),
        })
    }
}

impl Forge for OriginatedAddress {
    /// Forge originated(KT1) address.
    ///
    /// Doesn't add originated tag's prefix (1).
    fn forge(&self) -> Forged {
        Forged([self.as_ref().to_vec(), vec![0]].concat())
    }
}

impl Forge for Address {
    /// Forge implicit(tz1, tz2, tz3) or originated(KT1) address.
    ///
    /// Some fields that don't allow KT1 address, `ImplicitAddress::forge()`
    /// should be used instead as this adds extra tag prefix `0` to
    /// the output and the result will be invalid.
    fn forge(&self) -> Forged {
        Forged(match self {
            Address::Implicit(addr) => {
                [vec![0], addr.forge().take()].concat()
            }
            Address::Originated(addr) => {
                [vec![1], addr.forge().take()].concat()
            }
        })
    }
}

/// Address needs to be implicit(tz1, tz2, tz3). To delegate from originated(KT1) address
/// use [NewTransactionParameters::SetDelegate](crate::NewTransactionParameters::SetDelegate)
fn forge_delegate_addr(addr: &Option<ImplicitAddress>) -> Forged {
    Forged(match addr.as_ref() {
        Some(addr) => {
            [
                true.forge().take(),
                addr.forge().take(),
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
            self.source.forge().take(),
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
        let forged_parameters = match &self.parameters {
            Some(params) => [
                true.forge().take(),
                params.forge().take(),
            ].concat(),
            None => false.forge().take(),
        };

        Forged([
            OperationTag::Transaction.forge_nat().take(),
            self.source.forge().take(),
            self.fee.forge_nat().take(),
            self.counter.forge_nat().take(),
            self.gas_limit.forge_nat().take(),
            self.storage_limit.forge_nat().take(),
            self.amount.forge_nat().take(),
            self.destination.forge().take(),
            forged_parameters,
        ].concat())
    }
}

impl Forge for NewDelegationOperation {
    fn forge(&self) -> Forged {
        Forged([
            OperationTag::Delegation.forge_nat().take(),
            self.source.forge().take(),
            self.fee.forge_nat().take(),
            self.counter.forge_nat().take(),
            self.gas_limit.forge_nat().take(),
            self.storage_limit.forge_nat().take(),
            forge_delegate_addr(&self.delegate_to).take()
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
