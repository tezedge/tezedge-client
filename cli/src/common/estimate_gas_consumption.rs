use lib::NewOperationGroup;
use lib::api::RunOperation;
use lib::http_api::HttpApi;

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
    run_op_results: &Vec<serde_json::Value>,
) -> Option<u64> {
    run_op_results.iter()
        .find(|op| {
            op["kind"].as_str().map(|x| x == kind).unwrap_or(false)
        })
        .and_then(|op_result| {
            let result = &op_result["metadata"]["operation_result"];
            result["consumed_gas"].as_str()
        })
        .and_then(|x| x.parse().ok())

}

pub fn estimate_gas_consumption(
    op: &NewOperationGroup,
    api: &mut HttpApi,
) -> Result<OperationGroupGasConsumption, ()>
{
    let run_operation_result = api.run_operation(op)?;
    let op_results = run_operation_result["contents"]
        .as_array()
        .ok_or(())?;

    let reveal_gas = if let Some(_) = &op.reveal {
        Some(find_consumed_gas_for_kind("reveal", &op_results)
            .ok_or(())?)
    } else {
        None
    };

    let transaction_gas = if let Some(_) = &op.transaction {
        Some(find_consumed_gas_for_kind("transaction", &op_results)
            .ok_or(())? + 100) // TODO: add 100 outside of this method
    } else {
        None
    };

    let delegation_gas = if let Some(_) = &op.delegation {
        Some(find_consumed_gas_for_kind("delegation", &op_results)
            .ok_or(())?)
    } else {
        None
    };

    Ok(OperationGroupGasConsumption {
        reveal: reveal_gas,
        transaction: transaction_gas,
        delegation: delegation_gas,
    })
}
