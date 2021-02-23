use serde::{Serialize, Deserialize};

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
    pub amount: String,
    pub fee: String,
    pub counter: String,
    pub gas_limit: String,
    pub storage_limit: String,
}
