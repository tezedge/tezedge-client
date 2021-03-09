use serde::{Serialize, Deserialize};
use trezor_api::protos::{TezosSignTx_TezosContractID, TezosSignTx_TezosTransactionOp};

use crate::{
    Forge, NewTransactionParameters, Address, ImplicitAddress, OriginatedAddress,
    ImplicitOrOriginatedWithManager, OriginatedAddressWithManager,
};

#[derive(Debug, Clone)]
pub struct NewTransactionOperationBuilder {
    source: Option<ImplicitOrOriginatedWithManager>,
    destination: Option<Address>,
    amount: Option<u64>,
    fee: Option<u64>,
    counter: Option<u64>,
    gas_limit: Option<u64>,
    storage_limit: Option<u64>,
    /// Optional transaction parameters.
    ///
    /// Used to transfer/delegate from old (pre-Babylon) KT1 accounts.
    parameters: Option<NewTransactionParameters>
}

impl NewTransactionOperationBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn source<A>(mut self, source: A) -> Self
        where A: Into<ImplicitOrOriginatedWithManager>,
    {
        self.source = Some(source.into());
        self
    }

    pub fn destination(mut self, destination: Address) -> Self {
        self.destination = Some(destination);
        self
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = Some(amount);
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

    pub fn build(self) -> Result<NewTransactionOperation, ()> {
        let (source, destination, amount, fee, counter, gas_limit, storage_limit) = (
            self.source.ok_or(())?,
            self.destination.ok_or(())?,
            self.amount.ok_or(())?,
            self.fee.ok_or(())?,
            self.counter.ok_or(())?,
            self.gas_limit.ok_or(())?,
            self.storage_limit.ok_or(())?,
        );
        use ImplicitOrOriginatedWithManager::*;
        Ok(match source {
            Implicit(source) => {
                NewTransactionOperation {
                    source,
                    destination,
                    amount,
                    fee,
                    counter,
                    gas_limit,
                    storage_limit,
                    parameters: self.parameters,
                }
            }
            OriginatedWithManager(OriginatedAddressWithManager { address, manager }) => {
                NewTransactionOperation {
                    fee,
                    counter,
                    gas_limit,
                    storage_limit,
                    source: manager,
                    destination: address.into(),
                    amount: 0,
                    parameters: Some(NewTransactionParameters::Transfer {
                        to: destination,
                        amount: amount,
                    }),
                }
            }
        })
    }
}

impl Default for NewTransactionOperationBuilder {
    fn default() -> Self {
        Self {
            source: None,
            destination: None,
            amount: None,
            fee: None,
            counter: None,
            gas_limit: None,
            storage_limit: None,
            parameters: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewTransactionOperation {
    pub source: ImplicitAddress,
    pub destination: Address,
    #[serde(with = "crate::utils::serde_amount")]
    pub amount: u64,
    #[serde(with = "crate::utils::serde_amount")]
    pub fee: u64,
    #[serde(with = "crate::utils::serde_str")]
    pub counter: u64,
    #[serde(with = "crate::utils::serde_str")]
    pub gas_limit: u64,
    #[serde(with = "crate::utils::serde_str")]
    pub storage_limit: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<NewTransactionParameters>,
}

impl Into<TezosSignTx_TezosTransactionOp> for NewTransactionOperation {
    fn into(self) -> TezosSignTx_TezosTransactionOp {
        let mut new_tx = TezosSignTx_TezosTransactionOp::new();

        new_tx.set_source(self.source.forge().take());
        new_tx.set_destination(self.destination.into());
        new_tx.set_counter(self.counter);
        new_tx.set_fee(self.fee);
        new_tx.set_amount(self.amount);
        new_tx.set_gas_limit(self.gas_limit);
        new_tx.set_storage_limit(self.storage_limit);

        match self.parameters {
            Some(parameters) => {
                new_tx.set_parameters_manager(parameters.into());
            }
            None => {}
        };

        new_tx
    }
}
