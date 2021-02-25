//! # Error Handling

use std::result;

use protobuf::error::ProtobufError;

use crate::client::InteractionType;
use crate::{protos, transport};

/// Trezor error.
#[derive(Debug)]
pub enum Error {
	/// Less than one device was plugged in.
	NoDeviceFound,
	/// More than one device was plugged in.
	DeviceNotUnique,
	/// Transport error connecting to device.
	TransportConnect(transport::Error),
	/// Transport error while beginning a session.
	TransportBeginSession(transport::Error),
	/// Transport error while ending a session.
	TransportEndSession(transport::Error),
	/// Transport error while sending a message.
	TransportSendMessage(transport::Error),
	/// Transport error while receiving a message.
	TransportReceiveMessage(transport::Error),
	/// Received an unexpected message type from the device.
	UnexpectedMessageType(protos::MessageType), //TODO(stevenroose) type alias
	/// Error reading or writing protobuf messages.
	Protobuf(ProtobufError),
	/// A failure message was returned by the device.
	FailureResponse(protos::Failure),
	/// An unexpected interaction request was returned by the device.
	UnexpectedInteractionRequest(InteractionType),
	/// Provided entropy is not 32 bytes.
	InvalidEntropy,
	/// The device referenced a non-existing input or output index.
	TxRequestInvalidIndex(usize),
	/// The device referenced an unknown TXID.
	TxRequestUnknownTxid([u8; 32]),
	/// The PSBT is missing the full tx for given input.
	PsbtMissingInputTx([u8; 32]),
	// /// Device produced invalid TxRequest message.
	// MalformedTxRequest(protos::TxRequest),
	/// User provided invalid PSBT.
	InvalidPsbt(String),
}

impl From<ProtobufError> for Error {
	fn from(e: ProtobufError) -> Error {
		Error::Protobuf(e)
	}
}

/// Result type used in this crate.
pub type Result<T> = result::Result<T, Error>;
