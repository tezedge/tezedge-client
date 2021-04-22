use serde::Serialize;
use trezor_api::protos::TezosSignTx_TezosRevealOp;

use crate::{Forge, PublicKey, ImplicitAddress};
use utils::estimate_operation_fee;

#[derive(Serialize, Debug, Clone)]
pub struct NewRevealOperation {
    pub source: ImplicitAddress,
    pub public_key: PublicKey,
    #[serde(with = "utils::serde_amount")]
    pub fee: u64,
    #[serde(with = "utils::serde_str")]
    pub counter: u64,
    #[serde(with = "utils::serde_str")]
    pub gas_limit: u64,
    #[serde(with = "utils::serde_str")]
    pub storage_limit: u64,
}

impl NewRevealOperation {
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

impl Into<TezosSignTx_TezosRevealOp> for NewRevealOperation {
    /// Creates `TezosSignTx_TezosRevealOp`, protobuf type for Trezor.
    fn into(self) -> TezosSignTx_TezosRevealOp {
        let mut new_op = TezosSignTx_TezosRevealOp::new();

        new_op.set_source(self.source.forge().take());
        new_op.set_public_key(self.public_key.forge().take());
        new_op.set_counter(self.counter);
        new_op.set_fee(self.fee);
        new_op.set_gas_limit(self.gas_limit);
        new_op.set_storage_limit(self.storage_limit);

        new_op
    }
}
