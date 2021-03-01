use serde::{Serialize, Deserialize};
use trezor_api::protos::TezosSignTx_TezosDelegationOp;

use crate::PublicKeyHash;

#[derive(Debug, Clone)]
pub struct NewDelegationOperationBuilder {
    source: Option<PublicKeyHash>,
    delegate_to: Option<PublicKeyHash>,
    fee: Option<u64>,
    counter: Option<u64>,
    gas_limit: Option<u64>,
    storage_limit: Option<u64>,
}

impl NewDelegationOperationBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn source(mut self, source: PublicKeyHash) -> Self {
        self.source = Some(source);
        self
    }

    pub fn delegate_to(mut self, to: PublicKeyHash) -> Self {
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

    pub fn build(self) -> Result<NewDelegationOperation, ()> {
        // TODO: proper error handling
        Ok(NewDelegationOperation {
            source: self.source.unwrap(),
            delegate_to: self.delegate_to.unwrap(),
            fee: self.fee.unwrap(),
            counter: self.counter.unwrap(),
            gas_limit: self.gas_limit.unwrap(),
            storage_limit: self.storage_limit.unwrap(),
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
    pub source: PublicKeyHash,
    #[serde(rename = "delegate")]
    pub delegate_to: PublicKeyHash,
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

        let mut source: Vec<_> = self.source.into();
        // implicit public key hash prefix prefix
        source.insert(0, 0);

        let mut delegate_to: Vec<_> = self.delegate_to.into();
        // implicit public key hash prefix prefix
        delegate_to.insert(0, 0);

        new_op.set_source(source);
        new_op.set_delegate(delegate_to);

        new_op.set_counter(self.counter);
        new_op.set_fee(self.fee);
        new_op.set_gas_limit(self.gas_limit);
        new_op.set_storage_limit(self.storage_limit);

        new_op
    }
}
