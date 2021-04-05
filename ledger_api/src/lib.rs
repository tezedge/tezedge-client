use std::convert::TryInto;
use std::time::Duration;

use ledger_apdu::{APDUCommand, APDUAnswer, APDUErrorCodes};
use ledger::{TransportNativeHID, LedgerHIDError};

use types::{PUBLIC_KEY_LEN, PublicKey, ImplicitAddress};

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
    fn encode_path(path: &Vec<u32>) -> Vec<u8> {
        path.iter()
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
        path: Vec<u32>,
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

    /// Get Tezos address from Ledger for a given `path`(key derivation path).
    ///
    /// In reality we get public key from Ledger and then hash it.
    ///
    /// To avoid extra call to the ledger if public key is needed too,
    /// one can simply do:
    /// ```rust
    /// let public_key = ledger.get_public_key(path, prompt).unwrap();
    /// let address = public_key.hash();
    /// ```
    ///
    /// If `prompt` = `true`, user will be prompted on Ledger device, whether
    /// he/she wants to share public key for a given address to us.
    /// This functionality can be used to get address first without prompting,
    /// then verifying it by asking the user if address is same as shown in Ledger.
    pub fn get_address<'a>(
        &'a mut self,
        path: Vec<u32>,
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
        path: Vec<u32>,
        prompt: bool,
    ) -> LedgerResponse<'a, PublicKey>
    {
        self.public_key_request(path, prompt).send()
    }
}
