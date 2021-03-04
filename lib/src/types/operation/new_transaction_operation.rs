use serde::{Serialize, Deserialize};
use trezor_api::protos::{TezosSignTx_TezosContractID, TezosSignTx_TezosTransactionOp};

use crate::{Address, ImplicitAddress};

#[derive(Debug, Clone)]
pub struct NewTransactionOperationBuilder {
    source: Option<ImplicitAddress>,
    destination: Option<Address>,
    amount: Option<u64>,
    fee: Option<u64>,
    counter: Option<u64>,
    gas_limit: Option<u64>,
    storage_limit: Option<u64>,
}

impl NewTransactionOperationBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn source(mut self, source: ImplicitAddress) -> Self {
        self.source = Some(source);
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
        // TODO: proper error handling
        Ok(NewTransactionOperation {
            source: self.source.unwrap(),
            destination: self.destination.unwrap(),
            amount: self.amount.unwrap(),
            fee: self.fee.unwrap(),
            counter: self.counter.unwrap(),
            gas_limit: self.gas_limit.unwrap(),
            storage_limit: self.storage_limit.unwrap(),
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
}

impl Into<TezosSignTx_TezosTransactionOp> for NewTransactionOperation {
    fn into(self) -> TezosSignTx_TezosTransactionOp {
        let mut new_tx = TezosSignTx_TezosTransactionOp::new();

        let mut source: Vec<_> = self.source.into();
        // implicit public key hash prefix prefix
        source.insert(0, 0);

        let mut dest: Vec<_> = self.destination.into();
        // implicit public key hash prefix prefix
        dest.insert(0, 0);
        let mut destination = TezosSignTx_TezosContractID::new();
        destination.set_tag(trezor_api::protos::TezosSignTx_TezosContractID_TezosContractType::Implicit);
        destination.set_hash(dest);

        new_tx.set_source(source);
        new_tx.set_destination(destination);

        new_tx.set_counter(self.counter);
        new_tx.set_fee(self.fee);
        new_tx.set_amount(self.amount);
        new_tx.set_gas_limit(self.gas_limit);
        new_tx.set_storage_limit(self.storage_limit);

        new_tx
    }
}
