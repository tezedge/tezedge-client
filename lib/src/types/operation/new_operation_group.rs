use serde::Serialize;

use crate::BlockHash;
use super::{NewOperation, NewRevealOperation, NewTransactionOperation};

#[derive(Serialize)]
pub struct NewOperationGroup {
    branch: BlockHash,
    reveal: Option<NewRevealOperation>,
    transaction: Option<NewTransactionOperation>,
}

impl NewOperationGroup {
    pub fn new(branch: BlockHash) -> Self {
        Self {
            branch,
            reveal: None,
            transaction: None,
        }
    }

    /// Set reveal operation
    pub fn with_reveal(mut self, reveal: NewRevealOperation) -> Self {
        self.reveal = Some(reveal);
        self
    }

    /// Set transaction operation
    pub fn with_transaction(mut self, tx: NewTransactionOperation) -> Self {
        self.transaction = Some(tx);
        self
    }

    pub fn to_operations_vec(&self) -> Vec<NewOperation> {
        let reveal = self.reveal.clone().map(|x| x.into());
        let transaction = self.transaction.clone().map(|x| x.into());
        vec![reveal, transaction]
            .into_iter()
            .filter_map(|x| x)
            .collect()
    }
}
