use serde::{Serialize, Deserialize};
use trezor_api::protos::{TezosSignTx_TezosContractID, TezosSignTx_TezosTransactionOp};

use crate::PublicKeyHash;

#[derive(Debug, Clone)]
pub struct NewTransactionOperationBuilder {
    source: Option<PublicKeyHash>,
    destination: Option<PublicKeyHash>,
    amount: Option<String>,
    fee: Option<String>,
    counter: Option<String>,
    gas_limit: Option<String>,
    storage_limit: Option<String>,
}

impl NewTransactionOperationBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn source(mut self, source: PublicKeyHash) -> Self {
        self.source = Some(source);
        self
    }

    pub fn destination(mut self, destination: PublicKeyHash) -> Self {
        self.destination = Some(destination);
        self
    }

    pub fn amount(mut self, amount: String) -> Self {
        self.amount = Some(amount);
        self
    }

    pub fn fee(mut self, fee: String) -> Self {
        self.fee = Some(fee);
        self
    }

    pub fn counter(mut self, counter: String) -> Self {
        self.counter = Some(counter);
        self
    }

    pub fn gas_limit(mut self, gas_limit: String) -> Self {
        self.gas_limit = Some(gas_limit);
        self
    }

    pub fn storage_limit(mut self, storage_limit: String) -> Self {
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
    pub source: PublicKeyHash,
    pub destination: PublicKeyHash,
    // TODO: replace with u64
    pub amount: String,
    // TODO: replace with u64
    pub fee: String,
    // TODO: replace with u64
    pub counter: String,
    // TODO: replace with u64
    pub gas_limit: String,
    // TODO: replace with u64
    pub storage_limit: String,
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
        // let mut destination: Vec<_> = self.destination.into();
        // implicit public key hash prefix prefix
        // source.insert(0, 0);

        new_tx.set_source(source);
        new_tx.set_destination(destination);

        new_tx.set_counter(self.counter.parse().unwrap());
        new_tx.set_fee(self.fee.parse().unwrap());
        new_tx.set_amount(self.amount.parse().unwrap());
        new_tx.set_gas_limit(self.gas_limit.parse().unwrap());
        new_tx.set_storage_limit(self.storage_limit.parse().unwrap());

        new_tx
    }
}
