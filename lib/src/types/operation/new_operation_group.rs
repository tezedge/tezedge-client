use serde::Serialize;

use trezor_api::TezosSignTx;

use crate::BlockHash;
use super::{NewOperation, NewRevealOperation, NewTransactionOperation};

#[derive(Serialize, Debug, Clone)]
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

impl Into<TezosSignTx> for NewOperationGroup {
    /// Creates `TezosSignTx`
    ///
    /// **Warning**: make sure to set `address_n` field after, since
    /// it's required and not added here.
    fn into(self) -> TezosSignTx {
        let mut new_tx = TezosSignTx::new();
        new_tx.set_branch(self.branch.as_ref().to_vec());

        if let Some(reveal) = self.reveal {
            new_tx.set_reveal(reveal.into());
        }

        if let Some(tx) = self.transaction {
            new_tx.set_transaction(tx.into());
        }

        new_tx
    }
}
