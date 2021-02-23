use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Clone)]
pub struct NewTransactionOperationBuilder {
    source: Option<String>,
    destination: Option<String>,
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

    pub fn source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }

    pub fn destination(mut self, destination: String) -> Self {
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

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct NewTransactionOperation {
    pub source: String,
    pub destination: String,
    pub amount: String,
    pub fee: String,
    pub counter: String,
    pub gas_limit: String,
    pub storage_limit: String,
}
