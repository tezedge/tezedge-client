use serde::{Serialize, Deserialize};

use super::TransactionOperation;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Operation {
    Transaction(TransactionOperation),
}

impl Operation {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Operation::Transaction(_) => "transaction",
        }
    }
    // TODO: estimate fees and if estimate fee is bigger than max
    // allow fee, then show error to user.
}

impl From<TransactionOperation> for Operation {
    fn from(tx: TransactionOperation) -> Self {
        Self::Transaction(tx)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OperationWithKind {
    kind: String,

    #[serde(flatten)]
    operation: Operation,
}

impl From<Operation> for OperationWithKind {
    fn from(op: Operation) -> Self {
        Self {
            kind: op.kind_str().to_owned(),
            operation: op,
        }
    }
}
