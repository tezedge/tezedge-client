use std::fmt::{self, Display};
use std::convert::TryInto;

use ledger_apdu::{APDUCommand, APDUErrorCodes, map_apdu_error_description};
use ledger::{TransportNativeHID, LedgerHIDError};

use types::{PUBLIC_KEY_LEN, PublicKey, ImplicitAddress};

#[derive(thiserror::Error, Debug)]
pub enum LedgerError {
    Transport(#[from] LedgerHIDError),
    APDU(u16),
    InvalidDataLength,
}

impl Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(err) => err.fmt(f),
            Self::APDU(code) => {
                write!(f, "{}", map_apdu_error_description(*code))
            }
            Self::InvalidDataLength => {
                write!(f, "invalid data length received from ledger")
            }
        }
    }
}

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

    fn call(&self, command: &APDUCommand) -> Result<Vec<u8>, LedgerError> {
        let answer = self.transport.exchange(command)?;

        if (answer.retcode) != APDUErrorCodes::NoError as u16 {
            return Err(LedgerError::APDU(answer.retcode));
        }

        Ok(answer.data)
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
    pub fn get_address(
        &self,
        path: Vec<u32>,
        prompt: bool,
    ) -> Result<ImplicitAddress, LedgerError>
    {
        Ok(self.get_public_key(path, prompt)?.hash())
    }

    /// Get Tezos public key from Ledger for a given `path`(key derivation path).
    ///
    /// If `prompt` = `true`, user will be prompted on Ledger device, whether
    /// he/she wants to share public key for a given address to us.
    pub fn get_public_key(
        &self,
        path: Vec<u32>,
        prompt: bool,
    ) -> Result<PublicKey, LedgerError> {
        let path_bytes = Self::encode_path(&path);

        let bytes = self.call(&APDUCommand {
            cla: 0x80,
            ins: if prompt { 0x03 } else { 0x02 },
            p1: 0x00,
            p2: 0x00,
            data: [vec![path.len() as u8], path_bytes].concat(),
        })?;

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
    }
}
