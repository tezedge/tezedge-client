use serde::{Serialize, Serializer};
use trezor_api::protos::TezosSignTx_TezosOriginationOp;

use crate::{Forge, Forged, ImplicitAddress};
use crate::micheline::Micheline;
use utils::estimate_operation_fee;

pub const MANAGER_CONTRACT_CODE: &'static str = "02000000a005000764085e036c055f036d0000000325646f046c000000082564656661756c740501035d05020200000074037a072e020000005e0743036a00000313020000001e020000000403190325072c020000000002000000090200000004034f032705210002031e03540348020000001e020000000403190325072c020000000002000000090200000004034f0327034f0326034202000000080320053d036d0342";

#[derive(Debug, Clone)]
pub struct NewOriginationScript {
    // TODO: replace `Forged` type with Micheline.
    pub code: Forged,
    pub storage: Micheline,
}

// TODO: this is just a placeholder to make type system happy.
// remove after Serialization is implemented for `Micheline`.
impl Serialize for NewOriginationScript {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        s.serialize_none()
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct NewOriginationOperation {
    pub source: ImplicitAddress,
    #[serde(with = "utils::serde_amount")]
    pub balance: u64,
    #[serde(with = "utils::serde_amount")]
    pub fee: u64,
    #[serde(with = "utils::serde_str")]
    pub counter: u64,
    #[serde(with = "utils::serde_str")]
    pub gas_limit: u64,
    #[serde(with = "utils::serde_str")]
    pub storage_limit: u64,
    pub script: NewOriginationScript,
}

impl NewOriginationOperation {
    /// Estimate byte size of the operation.
    ///
    /// Forges the operation and counts bytes.
    pub fn estimate_bytes(&self) -> u64 {
        self.forge().take().len() as u64
    }

    /// Estimate minimal fee.
    pub fn estimate_fee(
        &self,
        base_fee: u64,
        ntez_per_byte: u64,
        ntez_per_gas: u64,
        estimated_gas: u64,
    ) -> u64 {
        estimate_operation_fee(
            base_fee,
            ntez_per_byte,
            ntez_per_gas,
            estimated_gas,
            self.estimate_bytes(),
        )
    }
}

impl Into<TezosSignTx_TezosOriginationOp> for NewOriginationOperation {
    /// Creates `TezosSignTx_TezosOriginationOp`, protobuf type for Trezor.
    fn into(self) -> TezosSignTx_TezosOriginationOp {
        let mut new_op = TezosSignTx_TezosOriginationOp::new();

        new_op.set_source(self.source.forge().take());
        new_op.set_counter(self.counter);
        new_op.set_fee(self.fee);
        new_op.set_balance(self.balance);
        new_op.set_gas_limit(self.gas_limit);
        new_op.set_storage_limit(self.storage_limit);
        new_op.set_script(self.script.forge().take());

        new_op
    }
}
