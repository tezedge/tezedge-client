use serde::Serialize;

use super::{NewRevealOperation, NewTransactionOperation, NewDelegationOperation};

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum NewOperation {
    Reveal(NewRevealOperation),
    Transaction(NewTransactionOperation),
    Delegation(NewDelegationOperation),
}

impl NewOperation {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Reveal(_) => "reveal",
            Self::Transaction(_) => "transaction",
            Self::Delegation(_) => "delegation",
        }
    }
    // TODO: estimate fees and if estimate fee is bigger than max
    // allow fee, then show error to user.
}

impl From<NewRevealOperation> for NewOperation {
    fn from(op: NewRevealOperation) -> Self {
        Self::Reveal(op)
    }
}

impl From<NewTransactionOperation> for NewOperation {
    fn from(op: NewTransactionOperation) -> Self {
        Self::Transaction(op)
    }
}

impl From<NewDelegationOperation> for NewOperation {
    fn from(op: NewDelegationOperation) -> Self {
        Self::Delegation(op)
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct NewOperationWithKind {
    kind: String,

    #[serde(flatten)]
    operation: NewOperation,
}

impl From<NewOperation> for NewOperationWithKind {
    fn from(op: NewOperation) -> Self {
        Self {
            kind: op.kind_str().to_owned(),
            operation: op,
        }
    }
}
