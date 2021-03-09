use serde::{Deserialize, Serialize};

use crate::{Forge, Address, ImplicitAddress};
use crate::trezor_api::protos::{
    TezosSignTx_TezosContractID,
    TezosSignTx_TezosTransactionOp_TezosParametersManager,
    TezosSignTx_TezosTransactionOp_TezosParametersManager_TezosManagerTransfer,
};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum NewTransactionParameters {
    SetDelegate(ImplicitAddress),
    CancelDelegate,
    Transfer {
        to: Address,
        amount: u64,
    },
}

impl Into<TezosSignTx_TezosTransactionOp_TezosParametersManager> for NewTransactionParameters {
    fn into(self) -> TezosSignTx_TezosTransactionOp_TezosParametersManager {
        let mut params = TezosSignTx_TezosTransactionOp_TezosParametersManager::new();

        match self {
            Self::SetDelegate(addr) => {
                params.set_set_delegate(addr.forge().take());
            }
            Self::CancelDelegate => {
                params.set_cancel_delegate(true);
            }
            Self::Transfer { to, amount } => {
                let mut transfer = TezosSignTx_TezosTransactionOp_TezosParametersManager_TezosManagerTransfer::new();
                transfer.set_destination(to.into());
                transfer.set_amount(amount);

                params.set_transfer(transfer);
            }
        }
        params
    }
}
