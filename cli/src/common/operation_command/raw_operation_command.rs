use std::fmt::{self, Display};
use console::style;
use dialoguer::theme::ColorfulTheme;

use lib::Address;
use lib::http_api::HttpApi;
use lib::utils::parse_float_amount;
use crate::common::operation_command::{OperationCommand, OperationOptions, OperationCommandState};
use crate::common::{parse_derivation_path, ParseDerivationPathError};

use super::{LedgerState, TrezorState};

pub fn ask_for_key_path() -> Result<String, std::io::Error> {
    // TODO: add cli argument to specify key_path there.
    eprintln!(
        "{} in order to create operation using trezor, you need to manually enter the {}, from which the {} was derived.\n\n      For more about key derivation path see: {}\n",
        style("help:").yellow(),
        style("path").green(),
        style("--from").bold(),
        style("https://learnmeabitcoin.com/technical/derivation-paths").cyan(),
    );
    dialoguer::Input::with_theme(&ColorfulTheme::default())
        .with_prompt("please enter a key derivation path")
        .with_initial_text("m/44'/1729'/0'")
        .interact_text()
}

#[derive(thiserror::Error, Debug)]
pub enum GetKeyPathError {
    // /// If operation is to be created using, but we don't have key_path
    // /// available to us and interactivity is turned off, this error will be thrown.
    // NotAvailable,
    IO(#[from] std::io::Error),
    ParseError(#[from] ParseDerivationPathError),
}

impl Display for GetKeyPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(err) => err.fmt(f),
            Self::ParseError(err) => err.fmt(f),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub struct ParseAddressError {
    pub kind: AddressKind,
    /// Input address as string before parsing.
    pub address: String,
    pub error: lib::FromPrefixedBase58CheckError,
}

#[derive(PartialEq, Debug, Clone)]
pub enum AddressKind {
    Source,
    Destination,
}

impl Display for ParseAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let field = match self.kind {
            AddressKind::Source => "--from",
            AddressKind::Destination => "--to",
        };

        write!(f,
            "invalid {} address: {}",
            style(field).bold(),
            style(&self.address).red(),
        )
    }
}

#[derive(thiserror::Error, Debug)]
pub struct InvalidApiTypeError(String);

impl Display for InvalidApiTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid api type: {}", style(&self.0).red())
    }
}

#[derive(thiserror::Error, Debug)]
pub struct InvalidFeeError(pub String);

impl Display for InvalidFeeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid fee: {}", style(&self.0).red())
    }
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub enum ParseOperationCommandError {
    IO(#[from] std::io::Error),

    InvalidApiType(#[from] InvalidApiTypeError),

    InvalidKeyDerivationPath(#[from] ParseDerivationPathError),

    InvalidAddress(#[from] ParseAddressError),

    InvalidFee(#[from] InvalidFeeError),

    #[error("interactivity is turned off, but `--key-path` wasn't passed in.")]
    MissingKeyPath
}

pub struct RawOptions {
    pub api_type: String,
    pub no_prompt: bool,
    pub use_trezor: bool,
    pub use_ledger: bool,
}

pub trait RawOperationCommand {
    fn get_raw_options(&self) -> RawOptions;
    fn get_api_endpoint(&self) -> String;
    fn get_raw_key_path(&self) -> Option<&str>;
    fn get_raw_from(&self) -> &str;
    fn get_raw_fee(&self) -> Option<&String>;

    fn parse(&self) -> Result<OperationCommand, ParseOperationCommandError> {
        let options = self.get_raw_options();
        let state = OperationCommandState::default();
        let mut trezor_state = None;
        let mut ledger_state = None;

        let api = match options.api_type.as_str() {
            "http" => Box::new(HttpApi::new(self.get_api_endpoint())),
            _ => Err(InvalidApiTypeError(options.api_type))?,
        };

        let from_is_key_path = self.get_raw_from().starts_with("m/");

        let key_path = if options.use_trezor || options.use_ledger {
            let raw_key_path = if from_is_key_path {
                self.get_raw_from().to_string()
            } else if let Some(key_path) = self.get_raw_key_path() {
                key_path.to_string()
            } else {
                if self.get_raw_options().no_prompt {
                    return Err(ParseOperationCommandError::MissingKeyPath);
                }
                ask_for_key_path()?
            };

            Some(parse_derivation_path(&raw_key_path)?)
        } else {
            None
        };

        if let Some(key_path) = key_path.clone() {
            if options.use_trezor {
                let trezor = crate::trezor::find_device_and_connect();
                trezor_state = Some(TrezorState { trezor, key_path });
            } else if options.use_ledger {
                let ledger = crate::ledger::find_device_and_connect();
                ledger_state = Some(LedgerState { ledger, key_path });
            } else {
                // key_path won't be set if neither `use_ledger` or `use_trezor` is set.
                unreachable!()
            }
        }

        let from = match (from_is_key_path, key_path) {
            (false, _) | (_, None) => {
                Address::from_base58check(self.get_raw_from())
                    .map_err(|err| ParseAddressError {
                        kind: AddressKind::Source,
                        error: err,
                        address: self.get_raw_from().to_string(),
                    })?
            }
            (true, Some(key_path)) => {
                if let Some(trezor_state) = trezor_state.as_mut() {
                    crate::trezor::get_address(&mut trezor_state.trezor, key_path)
                } else if let Some(ledger_state) = ledger_state.as_mut() {
                    crate::ledger::ledger_execute(
                        ledger_state.ledger.get_address(key_path, false),
                    )
                } else {
                    unreachable!()
                }.into()
            }
        };

        let fee = if let Some(raw_fee) = self.get_raw_fee() {
            Some(parse_float_amount(raw_fee)
                .map_err(|_| InvalidFeeError(raw_fee.to_string()))?)
        } else {
            None
        };

        Ok(OperationCommand {
            options: OperationOptions {
                no_prompt: self.get_raw_options().no_prompt,
            },
            from,
            fee,
            api,
            state,
            trezor_state,
            ledger_state,
            local_state: None,
        })
    }
}
