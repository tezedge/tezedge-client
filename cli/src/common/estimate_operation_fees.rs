use lib::NewOperationGroup;

use super::OperationGroupGasConsumption;

const BASE_FEE: u64 = 100;
const MIN_NTEZ_PER_GAS: u64 = 100;
const MIN_NTEZ_PER_BYTE: u64 = 1000;

#[derive(PartialEq, Debug, Clone)]
pub struct OperationFees {
    pub reveal: Option<u64>,
    pub transaction: Option<u64>,
    pub delegation: Option<u64>,
}

impl OperationFees {
    pub fn total(&self) -> u64 {
        self.reveal.unwrap_or(0)
            + self.transaction.unwrap_or(0)
            + self.delegation.unwrap_or(0)
    }
}

pub fn estimate_operation_fees(
    op: &NewOperationGroup,
    gas_consumption: &OperationGroupGasConsumption,
) -> OperationFees
{
    let reveal_fee = match (&op.reveal, gas_consumption.reveal) {
        (Some(op), Some(consumed_gas)) => {
            Some(op.estimate_fee(
                BASE_FEE,
                MIN_NTEZ_PER_BYTE,
                MIN_NTEZ_PER_GAS,
                consumed_gas,
            ))
        }
        _ => None,
    };

    let transaction_fee = match (&op.transaction, gas_consumption.transaction) {
        (Some(op), Some(consumed_gas)) => {
            Some(op.estimate_fee(
                BASE_FEE,
                MIN_NTEZ_PER_BYTE,
                MIN_NTEZ_PER_GAS,
                consumed_gas,
            ))
        }
        _ => None,
    };

    let delegation_fee = match (&op.delegation, gas_consumption.delegation) {
        (Some(op), Some(consumed_gas)) => {
            Some(op.estimate_fee(
                BASE_FEE,
                MIN_NTEZ_PER_BYTE,
                MIN_NTEZ_PER_GAS,
                consumed_gas,
            ))
        }
        _ => None,
    };

    OperationFees {
        reveal: reveal_fee.map(|x| x + 50),
        transaction: transaction_fee.map(|x| x + 100),
        delegation: delegation_fee.map(|x| x + 100),
    }
}
