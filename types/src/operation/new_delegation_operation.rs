use serde::{Serialize, Deserialize};
use trezor_api::protos::TezosSignTx_TezosDelegationOp;

use utils::estimate_operation_fee;
use crate::{Forge, ImplicitAddress, ImplicitOrOriginatedWithManager, OriginatedAddressWithManager};
use super::{NewOperation, NewTransactionOperation, NewTransactionParameters};

#[derive(Debug, Clone)]
pub struct NewDelegationOperationBuilder {
    pub source: ImplicitOrOriginatedWithManager,
    /// Optional.
    ///
    /// If set to `None`, account will stop delegating to anyone.
    pub delegate_to: Option<ImplicitAddress>,
    pub fee: u64,
    pub counter: u64,
    pub gas_limit: u64,
    pub storage_limit: u64,
}

impl NewDelegationOperationBuilder {
    pub fn build(self) -> NewOperation {
        use ImplicitOrOriginatedWithManager::*;
        match self.source {
            Implicit(source) => {
                NewOperation::Delegation(NewDelegationOperation {
                    source,
                    delegate_to: self.delegate_to,
                    fee: self.fee,
                    counter: self.counter,
                    gas_limit: self.gas_limit,
                    storage_limit: self.storage_limit,
                })
            }
            OriginatedWithManager(OriginatedAddressWithManager {
                address,
                manager,
            }) => {
                let parameters = match self.delegate_to {
                    Some(delegate) => NewTransactionParameters::SetDelegate(delegate),
                    None => NewTransactionParameters::CancelDelegate,
                };
                NewOperation::Transaction(NewTransactionOperation {
                    source: manager,
                    destination: address.into(),
                    fee: self.fee,
                    counter: self.counter,
                    gas_limit: self.gas_limit,
                    storage_limit: self.storage_limit,
                    amount: 0,
                    parameters: Some(parameters),
                })
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewDelegationOperation {
    pub source: ImplicitAddress,
    #[serde(rename = "delegate")]
    pub delegate_to: Option<ImplicitAddress>,
    #[serde(with = "utils::serde_amount")]
    pub fee: u64,
    #[serde(with = "utils::serde_str")]
    pub counter: u64,
    #[serde(with = "utils::serde_str")]
    pub gas_limit: u64,
    #[serde(with = "utils::serde_str")]
    pub storage_limit: u64,
}

impl NewDelegationOperation {
    pub fn estimate_bytes(&self) -> u64 {
        self.forge().take().len() as u64
    }

    pub fn estimate_fee(
        &self,
        base_fee: u64,
        ntez_per_byte: u64,
        ntez_per_gas: u64,
        estimated_gas: u64,
    ) -> u64 {
        estimate_operation_fee(
            base_fee,
            ntez_per_byte,
            ntez_per_gas,
            estimated_gas,
            self.estimate_bytes(),
        )
    }
}

impl Into<TezosSignTx_TezosDelegationOp> for NewDelegationOperation {
    fn into(self) -> TezosSignTx_TezosDelegationOp {
        let mut new_op = TezosSignTx_TezosDelegationOp::new();

        if let Some(delegate_to) = self.delegate_to {
            new_op.set_delegate(delegate_to.forge().take());
        }

        new_op.set_source(self.source.forge().take());
        new_op.set_counter(self.counter);
        new_op.set_fee(self.fee);
        new_op.set_gas_limit(self.gas_limit);
        new_op.set_storage_limit(self.storage_limit);

        new_op
    }
}
