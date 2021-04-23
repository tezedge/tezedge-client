//! Ledger Api
//!
//! Warning: Currently doesn't support getting the list of available devices.
//! It only supports connecting to the first available device, which
//! is what [Ledger::connect] does.
//!
//! # Example
//! ```no_run
//! # use ledger_api::Ledger;
//!
//! // connect to the first available ledger device.
//! let mut ledger = Ledger::connect().unwrap();
//!
//! // After this you can interact with Ledger device.
//!
//! // for example, get the address for key derivation path:
//! let path = "m/44'/1729'/0'/0'".parse().unwrap();
//! let address = ledger.get_address(&path, false).ack_all().unwrap();
//! ```
//!
//! # Interacting with Ledger
//!
//! On every call to Ledger, you will receive a [LedgerResponse]. As
//! you will notice based on type, you will receive:
//!
//! - [LedgerResponse::Ok(data)] meaning that command was successful
//!   and `data` will be whatever was requested from Ledger.
//! - [LedgerResponse::Err] meaning there was an error when executing
//!   our command.
//! - We also might receive some action request, like [LedgerResponse::RunAppRequest]
//!   which basically tells us that the user needs to confirm opening
//!   an application on the device and we need to wait for it.
//!
//!   Then we need to [RunAppRequest::ack] that request, which will trigger
//!   Ledger to show a prompt on the device screen. `ack` method will block,
//!   untill user interacts with the device.
//!
//!   As a response to the `ack`, we might receive another request,
//!   error or ok message.
//!
//!   With this architecture, we can first send a message to Ledger
//!   and after receiving response, if we get action request, before doing `ack`
//!   we can show the user on cli that he needs to confirm an action on the
//!   device. So that user won't have to guess why the cli is frozen and
//!   what it is waiting for.
//!
//!   **Note:** we don't actually receive requests from `Ledger` like we
//!   do with `Trezor`. Instead we interpret various error messages
//!   as a request to do some action.
//!
//!   For example if we receive error code that CLA is not supported,
//!   that means that **Tezos application** is not open, hence we can
//!   generate a request to run an application on the device.
//!

use std::convert::TryInto;
use std::time::Duration;

use ledger_apdu::{APDUCommand, APDUAnswer, APDUErrorCodes};
use ledger::{TransportNativeHID, LedgerHIDError};

use types::{PUBLIC_KEY_LEN, Forged, KeyDerivationPath, PublicKey, ImplicitAddress};
use crypto::{hex, blake2b, ToBase58Check, WithPrefix, Prefix};
use signer::OperationSignatureInfo;

mod ledger_error;
pub use ledger_error::{LedgerError, RunAppError, RunAppErrorKind};

mod retry_request;
pub use retry_request::RetryRequest;

mod reconnect_request;
pub use reconnect_request::ReconnectRequest;

mod unlock_request;
pub use unlock_request::UnlockRequest;

mod run_app_request;
pub use run_app_request::RunAppRequest;

mod ledger_request;
use ledger_request::{LedgerRequest, LedgerRequestData, ResultHandler};

mod ledger_response;
pub use ledger_response::LedgerResponse;

const TEZOS_APP_NAME: &'static str = "Tezos Wallet";
const TEZOS_CLA: u8 = 0x80;

pub struct Ledger {
    transport: TransportNativeHID,
}

impl Ledger {
    fn encode_path(path: &KeyDerivationPath) -> Vec<u8> {
        path.as_ref().iter()
            .flat_map(|x| x.to_be_bytes().to_vec())
            .collect()
    }

    /// Connect to first Ledger device.
    pub fn connect() -> Result<Self, LedgerError> {
        let transport = TransportNativeHID::new()?;

        Ok(Self {
            transport,
        })
    }

    /// Tries to reconnect to Ledger device every 200 millis for a
    /// number of `attempts`.
    pub(crate) fn reconnect(&mut self, attempts: usize) -> Result<(), LedgerHIDError> {
        for i in 1..=attempts {
            std::thread::sleep(Duration::from_millis(200));
            match TransportNativeHID::new() {
                Ok(transport) => {
                    self.transport = transport;
                    return Ok(());
                }
                Err(err) => {
                    if i == attempts {
                        return Err(err);
                    }
                }
            }
        }

        return Ok(());
    }

    pub(crate) fn raw_call(
        &mut self,
        command: &APDUCommand,
    ) -> Result<APDUAnswer, LedgerError> {
        Ok(self.transport.exchange(command)?)
    }

    pub(crate) fn call<'a, T: 'static>(
        &'a mut self,
        command: APDUCommand,
        handler: Box<ResultHandler<T>>,
    ) -> LedgerResponse<'a, T>
    {
        let answer = match self.raw_call(&command) {
            Ok(x) => x,
            Err(ledger_err) => {
                let req_data = LedgerRequestData { command, handler };
                if ledger_err.needs_reconnect() {
                    return ReconnectRequest::new(self, req_data).into();
                } else {
                    return ledger_err.into();
                }
            }
        };

        if answer.retcode != APDUErrorCodes::NoError as u16 {
            let req_data = LedgerRequestData { command, handler };

            if answer.retcode == APDUErrorCodes::ClaNotSupported as u16 {
                if req_data.command.cla == TEZOS_CLA {
                    return RunAppRequest::new(
                        self,
                        req_data,
                        TEZOS_APP_NAME,
                    ).into();
                }
            }

            let ledger_err = LedgerError::APDU(answer.retcode);

            if ledger_err.needs_unlock() {
                return UnlockRequest::new(self, req_data).into();
            }

            return ledger_err.into();
        }

        handler(self, answer.data).into()
    }

    /// Run an application on a Ledger device.
    ///
    /// ## Warning
    ///
    /// Might disconnect and reconnect Ledger from PC.
    pub fn run_app(&mut self, name: &str) -> Result<APDUAnswer, RunAppError> {
        let name_bytes = name.as_bytes().to_vec();

        if name_bytes.len() > u8::MAX as usize {
            return Err(RunAppError {
                name: name.to_string(),
                kind: RunAppErrorKind::NameTooLarge,
            });
        }

        let command = APDUCommand {
            cla: 0xE0,
            ins: 0xD8,
            p1: 0x00,
            p2: 0x00,
            data: name_bytes,
        };

        let result = self.transport
            .exchange(&command)
            .map_err(|err| RunAppError {
                name: name.to_string(),
                kind: err.into(),
            })?;

        if result.retcode == APDUErrorCodes::NoError as u16 {
            Ok(result)
        } else {
            Err(RunAppError {
                name: name.to_string(),
                kind: RunAppErrorKind::APDU(result.retcode),
            })
        }
    }


    fn public_key_request<'a>(
        &'a mut self,
        path: &KeyDerivationPath,
        prompt: bool,
    ) -> LedgerRequest<'a, PublicKey>
    {
        let path_bytes = Self::encode_path(&path);
        let command = APDUCommand {
            cla: TEZOS_CLA,
            ins: if prompt { 0x03 } else { 0x02 },
            p1: 0x00,
            p2: 0x00,
            data: [vec![path.len() as u8], path_bytes].concat(),
        };

        LedgerRequest::new(self, command)
            .map(|_, bytes| {
                let len = bytes[0] as usize;

                // len also counts in first byte, which specifies "curve".
                if len > bytes.len() + 1 || len - 1 != PUBLIC_KEY_LEN {
                    return Err(LedgerError::InvalidDataLength);
                }

                // TODO: implement for other curves.
                Ok(PublicKey::edpk(
                    // remove 2 byte prefix from the actual key.
                    // - first byte is for length of following public key.
                    // - second byte is curve type, right now we ignore it
                    //   as we only support edpk.
                    bytes[2..(PUBLIC_KEY_LEN + 2)].try_into()
                        .map_err(|_| LedgerError::InvalidDataLength)?
                ))
            })
    }

    fn sign_tx_request<'a>(
        &'a mut self,
        path: &KeyDerivationPath,
        // TODO: replace with ForgedOperation or Forged<NewOperationGroup>.
        forged_operation: Forged,
    ) -> LedgerRequest<'a, OperationSignatureInfo>
    {
        let path_bytes = Self::encode_path(&path);
        let initial_command = APDUCommand {
            cla: TEZOS_CLA,
            ins: 0x04,
            p1: 0x00,
            p2: 0x00,
            data: [vec![path.len() as u8], path_bytes].concat(),
        };

        let operation_bytes = forged_operation.as_ref().to_vec();

        LedgerRequest::new(self, initial_command)
            .map(move |ledger, _| {
                let encoded_op = [0x03].iter()
                    .chain(forged_operation.as_ref())
                    .map(|x| *x)
                    .collect::<Vec<_>>();

                let chunks = encoded_op.chunks(230).collect::<Vec<_>>();

                // TODO: change error type. This can only happen if
                // forged_operation is an empty array.
                let mut result = Err(LedgerError::InvalidDataLength);

                for (index, chunk) in chunks.iter().enumerate() {
                    let code = if index == chunks.len() - 1 {
                        0x81
                    } else {
                        0x01
                    };

                    let resp = ledger.call(APDUCommand {
                        cla: TEZOS_CLA,
                        ins: 0x04,
                        p1: code,
                        p2: 0x00,
                        data: chunk.to_vec(),
                    }, Box::new(|_, x| Ok(x)));

                    result = resp.ack_all();
                }

                result
            })
            .map(move |_, bytes| {
                if bytes.len() < 64 {
                    return Err(LedgerError::InvalidDataLength);
                }

                let signature_bytes = &bytes[0..64];

                let signature = signature_bytes
                    .with_prefix(Prefix::edsig)
                    .to_base58check();

                let operation_with_signature_bytes = [
                    operation_bytes.to_vec(),
                    signature_bytes.to_vec(),
                ].concat();

                let operation_with_signature = hex::encode(&operation_with_signature_bytes);

                let operation_hash = blake2b::digest_256(
                    &operation_with_signature_bytes,
                )
                    .with_prefix(Prefix::operation)
                    .to_base58check();

                Ok(OperationSignatureInfo {
                    signature,
                    operation_with_signature,
                    operation_hash,
                })
            })
    }

    /// Get Tezos address from Ledger for a given `path`(key derivation path).
    ///
    /// In reality we get public key from Ledger and then hash it.
    ///
    /// To avoid extra call to the ledger if public key is needed too,
    /// one can simply do:
    /// ```no_run
    /// # use ledger_api::Ledger;
    /// # let mut ledger = Ledger::connect().unwrap();
    /// let public_key = ledger.get_public_key(
    ///     &"m/44'/1729'/0'/0'".parse().unwrap(),
    ///     false,
    /// ).ack_all().unwrap();
    /// let address = public_key.hash();
    /// ```
    ///
    /// If `prompt` = `true`, user will be prompted on Ledger device, whether
    /// he/she wants to share public key for a given address to us.
    /// This functionality can be used to get address first without prompting,
    /// then verifying it by asking the user if address is same as shown in Ledger.
    pub fn get_address<'a>(
        &'a mut self,
        path: &KeyDerivationPath,
        prompt: bool,
    ) -> LedgerResponse<'a, ImplicitAddress>
    {
        self.public_key_request(path, prompt)
            .map(|_, pub_key| Ok(pub_key.hash()))
            .send()
    }

    /// Get Tezos public key from Ledger for a given `path`(key derivation path).
    ///
    /// If `prompt` = `true`, user will be prompted on Ledger device, whether
    /// he/she wants to share public key for a given address to us.
    pub fn get_public_key<'a>(
        &'a mut self,
        path: &KeyDerivationPath,
        prompt: bool,
    ) -> LedgerResponse<'a, PublicKey>
    {
        self.public_key_request(path, prompt).send()
    }

    pub fn sign_tx<'a>(
        &'a mut self,
        path: &KeyDerivationPath,
        // TODO: replace with ForgedOperation or Forged<NewOperationGroup>.
        forged_operation: Forged,
    ) -> LedgerResponse<'a, OperationSignatureInfo>
    {
        self.sign_tx_request(path, forged_operation).send()
    }
}
