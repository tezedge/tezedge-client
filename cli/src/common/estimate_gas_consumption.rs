use lib::NewOperationGroup;
use lib::api::{RunOperation, RunOperationError, RunOperationContents};

#[derive(PartialEq, Debug, Clone)]
pub struct OperationGroupGasConsumption {
    pub reveal: Option<u64>,
    pub transaction: Option<u64>,
    pub delegation: Option<u64>,
}

impl OperationGroupGasConsumption {
    pub fn total(&self) -> u64 {
        self.reveal.unwrap_or(0)
            + self.transaction.unwrap_or(0)
            + self.delegation.unwrap_or(0)
    }
}

fn find_consumed_gas_for_kind(
    kind: &str,
    run_op_contents: &RunOperationContents,
) -> Option<u64> {
    run_op_contents.iter()
        .find(|op| op.kind.as_str() == kind)
        .map(|op| op.consumed_gas)
}


pub fn estimate_gas_consumption<A>(
    op: &NewOperationGroup,
    api: &A,
) -> Result<OperationGroupGasConsumption, RunOperationError>
    where A: RunOperation + ?Sized,
{
    let op_results = api.run_operation(op)?;

    Ok(OperationGroupGasConsumption {
        reveal: find_consumed_gas_for_kind("reveal", &op_results),
        transaction: find_consumed_gas_for_kind("transaction", &op_results),
        delegation: find_consumed_gas_for_kind("delegation", &op_results),
    })
}
