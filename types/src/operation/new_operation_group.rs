use serde::Serialize;

use crate::BlockHash;
use super::{
    NewOperation,
    NewRevealOperation,
    NewTransactionOperation,
    NewDelegationOperation,
    NewOriginationOperation,
};

/// Group/Batch of Operations.
#[derive(Serialize, Debug, Clone)]
pub struct NewOperationGroup {
    pub branch: BlockHash,
    pub next_protocol_hash: String,
    pub reveal: Option<NewRevealOperation>,
    pub transaction: Option<NewTransactionOperation>,
    pub delegation: Option<NewDelegationOperation>,
    pub origination: Option<NewOriginationOperation>,
}

impl NewOperationGroup {
    pub fn new(branch: BlockHash, next_protocol_hash: String) -> Self {
        Self {
            branch,
            next_protocol_hash,
            reveal: None,
            transaction: None,
            delegation: None,
            origination: None,
        }
    }

    /// Set reveal operation
    pub fn with_reveal(mut self, op: NewRevealOperation) -> Self {
        self.reveal = Some(op);
        self
    }

    /// Set transaction operation
    pub fn with_transaction(mut self, op: NewTransactionOperation) -> Self {
        self.transaction = Some(op);
        self
    }

    /// Set delegation operation
    pub fn with_delegation(mut self, op: NewDelegationOperation) -> Self {
        self.delegation = Some(op);
        self
    }

    /// Set origination operation
    pub fn with_origination(mut self, op: NewOriginationOperation) -> Self {
        self.origination = Some(op);
        self
    }

    pub fn with_operation<T>(self, op: T) -> Self
        where T: Into<NewOperation>,
    {
        match op.into() {
            NewOperation::Reveal(op) => self.with_reveal(op),
            NewOperation::Transaction(op) => self.with_transaction(op),
            NewOperation::Delegation(op) => self.with_delegation(op),
            NewOperation::Origination(op) => self.with_origination(op),
        }
    }

    pub fn to_operations_vec(&self) -> Vec<NewOperation> {
        let reveal = self.reveal.clone().map(|x| x.into());
        let transaction = self.transaction.clone().map(|x| x.into());
        let delegation = self.delegation.clone().map(|x| x.into());
        let origination = self.origination.clone().map(|x| x.into());
        vec![reveal, transaction, delegation, origination]
            .into_iter()
            .filter_map(|x| x)
            .collect()
    }
}
