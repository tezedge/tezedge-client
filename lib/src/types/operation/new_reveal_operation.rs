use serde::Serialize;
use trezor_api::protos::TezosSignTx_TezosRevealOp;

use crate::{PublicKey, ImplicitAddress};

#[derive(Debug, Clone)]
pub struct NewRevealOperationBuilder {
    source: Option<ImplicitAddress>,
    public_key: Option<PublicKey>,
    fee: Option<u64>,
    counter: Option<u64>,
    gas_limit: Option<u64>,
    storage_limit: Option<u64>,
}

impl NewRevealOperationBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn source(mut self, source: ImplicitAddress) -> Self {
        self.source = Some(source);
        self
    }

    pub fn public_key(mut self, public_key: PublicKey) -> Self {
        self.public_key = Some(public_key);
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
    pub source: ImplicitAddress,
    pub public_key: PublicKey,
    #[serde(with = "crate::utils::serde_amount")]
    pub fee: u64,
    #[serde(with = "crate::utils::serde_str")]
    pub counter: u64,
    #[serde(with = "crate::utils::serde_str")]
    pub gas_limit: u64,
    #[serde(with = "crate::utils::serde_str")]
    pub storage_limit: u64,
}

impl Into<TezosSignTx_TezosRevealOp> for NewRevealOperation {
    fn into(self) -> TezosSignTx_TezosRevealOp {
        let mut new_op = TezosSignTx_TezosRevealOp::new();

        let mut source: Vec<_> = self.source.into();
        // implicit public key prefix prefix
        source.insert(0, 0);

        let mut public_key = self.public_key.as_ref().to_vec();
        public_key.insert(0, 0);

        new_op.set_source(source);
        new_op.set_public_key(public_key);
        new_op.set_counter(self.counter);
        new_op.set_fee(self.fee);
        new_op.set_gas_limit(self.gas_limit);
        new_op.set_storage_limit(self.storage_limit);

        new_op
    }
}
