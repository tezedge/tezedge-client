use serde::{Deserialize, Serialize, Serializer};
use serde_json::json;
use sodiumoxide::hex;

use crate::{Forge, Address, ImplicitAddress};
use trezor_api::protos::{
    TezosSignTx_TezosTransactionOp_TezosParametersManager,
    TezosSignTx_TezosTransactionOp_TezosParametersManager_TezosManagerTransfer,
};

/// Parameters for Smart Contract.
///
/// In order to interact with the Smart Contract, transaction must be
/// created with the destination set to smart contract's address.
///
/// Note: Smart contract's address starts with **KT1**.
#[derive(Deserialize, PartialEq, Debug, Clone)]
pub enum NewTransactionParameters {
    /// Set delegate of the smart contract.
    SetDelegate(ImplicitAddress),
    /// Cancel/withdraw active delegation of the smart contract.
    CancelDelegate,
    /// Transfer funds from smart contract to another address.
    Transfer {
        /// Address to transfer funds to.
        to: Address,
        /// Amount to transfer.
        amount: u64,
    },
}

fn set_delegate_json(addr: &ImplicitAddress) -> serde_json::Value {
    let delegate = hex::encode(addr.forge().take());

    json!([
        { "prim": "DROP" },
        { "prim": "NIL", "args": [{ "prim": "operation" }] },
        { "prim": "PUSH", "args": [
            { "prim": "key_hash" },
            { "bytes": delegate },
        ] },
        { "prim": "SOME" },
        { "prim": "SET_DELEGATE" },
        { "prim": "CONS" }
    ])
}

fn cancel_delegate_json() -> serde_json::Value {
    json!([
        { "prim": "DROP" },
        { "prim": "NIL", "args": [{ "prim": "operation" }] },
        { "prim": "NONE", "args": [{ "prim": "key_hash" }] },
        { "prim": "SET_DELEGATE" },
        { "prim": "CONS" }
    ])
}

fn transfer_json(to: &Address, amount: u64) -> serde_json::Value {
    let amount = amount.to_string();

    match to {
        Address::Implicit(destination) => json!([
            { "prim": "DROP" },
            { "prim": "NIL", "args": [{ "prim": "operation" }] },
            { "prim": "PUSH", "args": [
                { "prim": "key_hash" },
                { "bytes": hex::encode(destination.forge().take()) },
            ] },
            { "prim": "IMPLICIT_ACCOUNT" },
            { "prim": "PUSH", "args": [
                { "prim": "mutez" },
                { "int": amount },
            ] },
            { "prim": "UNIT" },
            { "prim": "TRANSFER_TOKENS" },
            { "prim": "CONS" },
        ]),
        Address::Originated(_) => json!([
            { "prim": "DROP" },
            { "prim": "NIL", "args": [{ "prim": "operation" }] },
            { "prim": "PUSH", "args": [
                { "prim": "address" },
                { "bytes": hex::encode(to.forge().take()) },
            ] },
            { "prim": "CONTRACT", "args": [{ "prim": "unit" }] },
            [ { "prim": "IF_NONE", "args": [
                [[{ "prim": "UNIT" }, { "prim": "FAILWITH" }]],
                [],
            ] } ],
            { "prim": "PUSH", "args": [
                { "prim": "mutez" },
                { "int": amount },
            ] },
            { "prim": "UNIT" },
            { "prim": "TRANSFER_TOKENS" },
            { "prim": "CONS" },
        ])
    }
}

impl Serialize for NewTransactionParameters {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
    {
        json!({
            "entrypoint": "do",
            "value": match self {
                Self::SetDelegate(addr) => set_delegate_json(addr),
                Self::CancelDelegate => cancel_delegate_json(),
                Self::Transfer { to, amount } => transfer_json(to, *amount),
            },
        }).serialize(s)
    }
}

impl Into<TezosSignTx_TezosTransactionOp_TezosParametersManager> for NewTransactionParameters {
    /// Creates `TezosSignTx_TezosTransactionOp_TezosParametersManager`, protobuf type for Trezor.
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
