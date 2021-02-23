use serde::{Serialize, Deserialize};

use super::NewTransactionOperation;

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum NewOperation {
    Transaction(NewTransactionOperation),
}

impl NewOperation {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Transaction(_) => "transaction",
        }
    }
    // TODO: estimate fees and if estimate fee is bigger than max
    // allow fee, then show error to user.
}

impl From<NewTransactionOperation> for NewOperation {
    fn from(tx: NewTransactionOperation) -> Self {
        Self::Transaction(tx)
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
