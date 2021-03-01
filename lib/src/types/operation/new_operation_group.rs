use serde::Serialize;

use trezor_api::TezosSignTx;

use crate::BlockHash;
use super::{NewOperation, NewRevealOperation, NewTransactionOperation, NewDelegationOperation};

#[derive(Serialize, Debug, Clone)]
pub struct NewOperationGroup {
    pub branch: BlockHash,
    pub next_protocol_hash: String,
    pub reveal: Option<NewRevealOperation>,
    pub transaction: Option<NewTransactionOperation>,
    pub delegation: Option<NewDelegationOperation>,
}

impl NewOperationGroup {
    pub fn new(branch: BlockHash, next_protocol_hash: String) -> Self {
        Self {
            branch,
            next_protocol_hash,
            reveal: None,
            transaction: None,
            delegation: None,
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

    pub fn to_operations_vec(&self) -> Vec<NewOperation> {
        let reveal = self.reveal.clone().map(|x| x.into());
        let transaction = self.transaction.clone().map(|x| x.into());
        let delegation = self.delegation.clone().map(|x| x.into());
        vec![reveal, transaction, delegation]
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

        if let Some(op) = self.reveal {
            new_tx.set_reveal(op.into());
        }

        if let Some(op) = self.transaction {
            new_tx.set_transaction(op.into());
        }

        if let Some(op) = self.delegation {
            new_tx.set_delegation(op.into());
        }

        new_tx
    }
}
