use serde::{Serialize, Deserialize};

use crate::PublicKey;

#[derive(Debug, Clone)]
pub struct NewRevealOperationBuilder {
    source: Option<String>,
    public_key: Option<PublicKey>,
    fee: Option<String>,
    counter: Option<String>,
    gas_limit: Option<String>,
    storage_limit: Option<String>,
}

impl NewRevealOperationBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }

    pub fn public_key(mut self, public_key: PublicKey) -> Self {
        self.public_key = Some(public_key);
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

    pub fn build(self) -> Result<NewRevealOperation, ()> {
        // TODO: proper error handling
        Ok(NewRevealOperation {
            source: self.source.unwrap(),
            public_key: self.public_key.unwrap(),
            fee: self.fee.unwrap(),
            counter: self.counter.unwrap(),
            gas_limit: self.gas_limit.unwrap(),
            storage_limit: self.storage_limit.unwrap(),
        })
    }
}

impl Default for NewRevealOperationBuilder {
    fn default() -> Self {
        Self {
            source: None,
            public_key: None,
            fee: None,
            counter: None,
            gas_limit: None,
            storage_limit: None,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct NewRevealOperation {
    pub source: String,
    pub public_key: PublicKey,
    pub fee: String,
    pub counter: String,
    pub gas_limit: String,
    pub storage_limit: String,
}
