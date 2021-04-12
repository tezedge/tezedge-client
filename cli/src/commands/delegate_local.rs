use std::fmt::{self, Display};
use structopt::StructOpt;
use console::style;

use lib::utils::parse_float_amount;
use lib::http_api::HttpApi;
use lib::{Address, PrivateKey, PublicKey};

use crate::commands::CommandError;
use crate::common::operation_command::*;

#[derive(thiserror::Error, Debug)]
pub struct ParseKeyError {
    kind: KeyKind,
    /// Input address as string before parsing.
    key: String,
    error: lib::FromPrefixedBase58CheckError,
}

#[derive(PartialEq, Debug, Clone)]
pub enum KeyKind {
    Public,
    Private,
}

impl Display for ParseKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let field = match self.kind {
            KeyKind::Public => "--public-key",
            KeyKind::Private => "--private-key",
        };

        write!(f,
            "invalid {}: {}",
            style(field).bold(),
            style(&self.key).red(),
        )
    }
}

/// Delegate balance to baker
#[derive(StructOpt)]
pub struct DelegateLocal {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    /// Disable interactivity and accept default answers to prompts.
    #[structopt(short = "y", long = "no-prompt")]
    pub no_prompt: bool,

    #[structopt(short = "E", long)]
    pub endpoint: String,

    #[structopt(long = "public-key")]
    pub public_key: String,

    #[structopt(long = "private-key")]
    pub private_key: String,

    #[structopt(short, long)]
    pub from: String,

    #[structopt(short, long)]
    pub to: String,

    /// Specify fee for the delegation.
    ///
    /// If not specified, fee will be estimated and you will be prompted
    /// whether or not you accept estimate or would like to enter custom one.
    #[structopt(long)]
    pub fee: Option<String>,
}

impl DelegateLocal {
    fn public_key(&self) -> Result<PublicKey, ParseKeyError> {
        PublicKey::from_base58check(&self.public_key)
            .map_err(|error| ParseKeyError {
                error,
                kind: KeyKind::Public,
                key: self.public_key.to_string(),
            })
    }

    fn private_key(&self) -> Result<PrivateKey, ParseKeyError> {
        PrivateKey::from_base58check(&self.private_key)
            .map_err(|error| ParseKeyError {
                error,
                kind: KeyKind::Private,
                key: self.private_key.to_string(),
            })
    }

    fn fee(&self) -> Result<Option<u64>, InvalidFeeError> {
        if let Some(raw_fee) = self.fee.as_ref() {
            Ok(Some(parse_float_amount(raw_fee)
                .map_err(|_| InvalidFeeError(raw_fee.to_string()))?))
        } else {
            Ok(None)
        }
    }

    pub fn execute(self) -> Result<(), CommandError> {
        let public_key = self.public_key()?;
        let private_key = self.private_key()?;

        Ok(OperationCommand {
            options: OperationOptions {
                no_prompt: self.no_prompt,
            },
            api: Box::new(HttpApi::new(self.endpoint.clone())),
            from: public_key.hash().into(),
            to: Address::from_base58check(&self.to)
                .map_err(|error| ParseAddressError {
                    error,
                    kind: AddressKind::Destination,
                    address: self.to.to_string(),
                })?,
            fee: self.fee()?,
            state: Default::default(),
            trezor_state: None,
            ledger_state: None,
            local_state: Some(LocalWalletState { public_key, private_key }),
        }.delegate()?)
    }
}
