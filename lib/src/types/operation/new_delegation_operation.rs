use serde::{Serialize, Deserialize};
use trezor_api::protos::TezosSignTx_TezosDelegationOp;

use crate::{Forge, ImplicitAddress, ImplicitOrOriginatedWithManager, OriginatedAddressWithManager};
use super::{NewOperation, NewTransactionOperation, NewTransactionParameters};

#[derive(Debug, Clone)]
pub struct NewDelegationOperationBuilder {
    source: Option<ImplicitOrOriginatedWithManager>,
    /// Optional.
    ///
    /// If set to `None`, account will stop delegating to anyone.
    delegate_to: Option<ImplicitAddress>,
    fee: Option<u64>,
    counter: Option<u64>,
    gas_limit: Option<u64>,
    storage_limit: Option<u64>,
}

impl NewDelegationOperationBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn source<A>(mut self, source: A) -> Self
        where A: Into<ImplicitOrOriginatedWithManager>,
    {
        self.source = Some(source.into());
        self
    }

    /// If not set, currently active delegation will be canceled.
    pub fn delegate_to(mut self, to: ImplicitAddress) -> Self {
        self.delegate_to = Some(to);
        self
    }

    pub fn fee(mut self, fee: u64) -> Self {
        self.fee = Some(fee);
        self
    }

    pub fn counter(mut self, counter: u64) -> Self {
        self.counter = Some(counter);
        self
    }

    pub fn gas_limit(mut self, gas_limit: u64) -> Self {
        self.gas_limit = Some(gas_limit);
        self
    }

    pub fn storage_limit(mut self, storage_limit: u64) -> Self {
        self.storage_limit = Some(storage_limit);
        self
    }

    pub fn build(self) -> Result<NewOperation, ()> {
        let (source, delegate_to, fee, counter, gas_limit, storage_limit) = (
            self.source.ok_or(())?,
            self.delegate_to,
            self.fee.ok_or(())?,
            self.counter.ok_or(())?,
            self.gas_limit.ok_or(())?,
            self.storage_limit.ok_or(())?,
        );
        use ImplicitOrOriginatedWithManager::*;
        Ok(match source {
            Implicit(source) => {
                NewOperation::Delegation(NewDelegationOperation {
                    source,
                    delegate_to,
                    fee,
                    counter,
                    gas_limit,
                    storage_limit,
                })
            }
            OriginatedWithManager(OriginatedAddressWithManager {
                address,
                manager,
            }) => {
                let parameters = match delegate_to {
                    Some(delegate) => NewTransactionParameters::SetDelegate(delegate),
                    None => NewTransactionParameters::CancelDelegate,
                };
                NewOperation::Transaction(NewTransactionOperation {
                    fee,
                    counter,
                    gas_limit,
                    storage_limit,
                    source: manager,
                    destination: address.into(),
                    amount: 0,
                    parameters: Some(parameters),
                })
            }
        })
    }
}

impl Default for NewDelegationOperationBuilder {
    fn default() -> Self {
        Self {
            source: None,
            delegate_to: None,
            fee: None,
            counter: None,
            gas_limit: None,
            storage_limit: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewDelegationOperation {
    pub source: ImplicitAddress,
    #[serde(rename = "delegate")]
    pub delegate_to: Option<ImplicitAddress>,
    #[serde(with = "crate::utils::serde_amount")]
    pub fee: u64,
    #[serde(with = "crate::utils::serde_str")]
    pub counter: u64,
    #[serde(with = "crate::utils::serde_str")]
    pub gas_limit: u64,
    #[serde(with = "crate::utils::serde_str")]
    pub storage_limit: u64,
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
