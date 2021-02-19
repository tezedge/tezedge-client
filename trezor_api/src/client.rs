use std::fmt;

use sodiumoxide::hex;

use crate::transport::{ProtoMessage, Transport};
use crate::messages::TrezorMessage;
use super::{protos, TrezorModel, Error, Result};
use protos::{MessageType::*, TezosAddress, TezosSignTx, TezosSignedTx};

// Some types with raw protos that we use in the public interface so they have to be exported.
pub use protos::ButtonRequest_ButtonRequestType as ButtonRequestType;
pub use protos::Features;
pub use protos::PinMatrixRequest_PinMatrixRequestType as PinMatrixRequestType;

/// The different options for the number of words in a seed phrase.
pub enum WordCount {
	W12 = 12,
	W18 = 18,
	W24 = 24,
}

/// The different types of user interactions the Trezor device can request.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum InteractionType {
	Button,
	PinMatrix,
	Passphrase,
	PassphraseState,
}

//TODO(stevenroose) should this be FnOnce and put in an FnBox?
/// Function to be passed to the `Trezor.call` method to process the Trezor response message into a
/// general-purpose type.
pub type ResultHandler<'a, T, R> = Fn(&'a mut Trezor, R) -> Result<T>;

/// A button request message sent by the device.
pub struct ButtonRequest<'a, T, R: TrezorMessage> {
	message: protos::ButtonRequest,
	client: &'a mut Trezor,
	result_handler: Box<ResultHandler<'a, T, R>>,
}

impl<'a, T, R: TrezorMessage> fmt::Debug for ButtonRequest<'a, T, R> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&self.message, f)
	}
}

impl<'a, T, R: TrezorMessage> ButtonRequest<'a, T, R> {
	/// The type of button request.
	pub fn request_type(&self) -> ButtonRequestType {
		self.message.get_code()
	}

	/// The metadata sent with the button request.
	pub fn request_data(&self) -> &str {
		self.message.get_data()
	}

	/// Ack the request and get the next message from the device.
	pub fn ack(self) -> Result<TrezorResponse<'a, T, R>> {
		let req = protos::ButtonAck::new();
		self.client.call(req, self.result_handler)
	}
}

/// A PIN matrix request message sent by the device.
pub struct PinMatrixRequest<'a, T, R: TrezorMessage> {
	message: protos::PinMatrixRequest,
	client: &'a mut Trezor,
	result_handler: Box<ResultHandler<'a, T, R>>,
}

impl<'a, T, R: TrezorMessage> fmt::Debug for PinMatrixRequest<'a, T, R> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&self.message, f)
	}
}

impl<'a, T, R: TrezorMessage> PinMatrixRequest<'a, T, R> {
	/// The type of PIN matrix request.
	pub fn request_type(&self) -> PinMatrixRequestType {
		self.message.get_field_type()
	}

	/// Ack the request with a PIN and get the next message from the device.
	pub fn ack_pin(self, pin: String) -> Result<TrezorResponse<'a, T, R>> {
		let mut req = protos::PinMatrixAck::new();
		req.set_pin(pin);
		self.client.call(req, self.result_handler)
	}
}

/// A passphrase request message sent by the device.
pub struct PassphraseRequest<'a, T, R: TrezorMessage> {
	message: protos::PassphraseRequest,
	client: &'a mut Trezor,
	result_handler: Box<ResultHandler<'a, T, R>>,
}

impl<'a, T, R: TrezorMessage> fmt::Debug for PassphraseRequest<'a, T, R> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&self.message, f)
	}
}

impl<'a, T, R: TrezorMessage> PassphraseRequest<'a, T, R> {
	/// Check whether the use is supposed to enter the passphrase on the device or not.
	pub fn on_device(&self) -> bool {
		self.message.get_on_device()
	}

	/// Ack the request with a passphrase and get the next message from the device.
	pub fn ack_passphrase(self, passphrase: String) -> Result<TrezorResponse<'a, T, R>> {
		let mut req = protos::PassphraseAck::new();
		req.set_passphrase(passphrase);
		self.client.call(req, self.result_handler)
	}

	/// Ack the request without a passphrase to let the user enter it on the device
	/// and get the next message from the device.
	pub fn ack(self) -> Result<TrezorResponse<'a, T, R>> {
		let req = protos::PassphraseAck::new();
		self.client.call(req, self.result_handler)
	}
}

/// A passphrase state request message sent by the device.
pub struct PassphraseStateRequest<'a, T, R: TrezorMessage> {
	message: protos::PassphraseStateRequest,
	client: &'a mut Trezor,
	result_handler: Box<ResultHandler<'a, T, R>>,
}

impl<'a, T, R: TrezorMessage> fmt::Debug for PassphraseStateRequest<'a, T, R> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&self.message, f)
	}
}

impl<'a, T, R: TrezorMessage> PassphraseStateRequest<'a, T, R> {
	/// The passphrase state provided by the device.
	pub fn passphrase_state(&self) -> &[u8] {
		self.message.get_state()
	}

	/// Ack the receipt of the passphrase state.
	pub fn ack(self) -> Result<TrezorResponse<'a, T, R>> {
		let req = protos::PassphraseStateAck::new();
		self.client.call(req, self.result_handler)
	}
}

/// A response from a Trezor device.  On every message exchange, instead of the expected/desired
/// response, the Trezor can ask for some user interaction, or can send a failure.
#[derive(Debug)]
pub enum TrezorResponse<'a, T, R: TrezorMessage> {
	Ok(T),
	Failure(protos::Failure),
	ButtonRequest(ButtonRequest<'a, T, R>),
	PinMatrixRequest(PinMatrixRequest<'a, T, R>),
	PassphraseRequest(PassphraseRequest<'a, T, R>),
	//TODO(stevenroose) This should be taken out of this enum and intrinsically attached to the
	// PassphraseRequest variant.  However, it's currently impossible to do this.  It might be
	// possible to do with FnBox (currently nightly) or when Box<FnOnce> becomes possible.
	PassphraseStateRequest(PassphraseStateRequest<'a, T, R>),
}

impl<'a, T, R: TrezorMessage> fmt::Display for TrezorResponse<'a, T, R> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			TrezorResponse::Ok(ref _m) => write!(f, "Ok"), //TODO(stevenroose) should we make T: Debug?
			TrezorResponse::Failure(ref m) => write!(f, "Failure: {:?}", m),
			TrezorResponse::ButtonRequest(ref r) => write!(f, "ButtonRequest: {:?}", r),
			TrezorResponse::PinMatrixRequest(ref r) => write!(f, "PinMatrixRequest: {:?}", r),
			TrezorResponse::PassphraseRequest(ref r) => write!(f, "PassphraseRequest: {:?}", r),
			TrezorResponse::PassphraseStateRequest(ref r) => {
				write!(f, "PassphraseStateRequest: {:?}", r)
			}
		}
	}
}

impl<'a, T, R: TrezorMessage> TrezorResponse<'a, T, R> {
	/// Get the actual `Ok` response value or an error if not `Ok`.
	pub fn ok(self) -> Result<T> {
		match self {
			TrezorResponse::Ok(m) => Ok(m),
			TrezorResponse::Failure(m) => Err(Error::FailureResponse(m)),
			TrezorResponse::ButtonRequest(_) => {
				Err(Error::UnexpectedInteractionRequest(InteractionType::Button))
			}
			TrezorResponse::PinMatrixRequest(_) => {
				Err(Error::UnexpectedInteractionRequest(InteractionType::PinMatrix))
			}
			TrezorResponse::PassphraseRequest(_) => {
				Err(Error::UnexpectedInteractionRequest(InteractionType::Passphrase))
			}
			TrezorResponse::PassphraseStateRequest(_) => {
				Err(Error::UnexpectedInteractionRequest(InteractionType::PassphraseState))
			}
		}
	}

	/// Get the button request object or an error if not `ButtonRequest`.
	pub fn button_request(self) -> Result<ButtonRequest<'a, T, R>> {
		match self {
			TrezorResponse::ButtonRequest(r) => Ok(r),
			TrezorResponse::Ok(_) => Err(Error::UnexpectedMessageType(R::message_type())),
			TrezorResponse::Failure(m) => Err(Error::FailureResponse(m)),
			TrezorResponse::PinMatrixRequest(_) => {
				Err(Error::UnexpectedInteractionRequest(InteractionType::PinMatrix))
			}
			TrezorResponse::PassphraseRequest(_) => {
				Err(Error::UnexpectedInteractionRequest(InteractionType::Passphrase))
			}
			TrezorResponse::PassphraseStateRequest(_) => {
				Err(Error::UnexpectedInteractionRequest(InteractionType::PassphraseState))
			}
		}
	}

	/// Get the PIN matrix request object or an error if not `PinMatrixRequest`.
	pub fn pin_matrix_request(self) -> Result<PinMatrixRequest<'a, T, R>> {
		match self {
			TrezorResponse::PinMatrixRequest(r) => Ok(r),
			TrezorResponse::Ok(_) => Err(Error::UnexpectedMessageType(R::message_type())),
			TrezorResponse::Failure(m) => Err(Error::FailureResponse(m)),
			TrezorResponse::ButtonRequest(_) => {
				Err(Error::UnexpectedInteractionRequest(InteractionType::Button))
			}
			TrezorResponse::PassphraseRequest(_) => {
				Err(Error::UnexpectedInteractionRequest(InteractionType::Passphrase))
			}
			TrezorResponse::PassphraseStateRequest(_) => {
				Err(Error::UnexpectedInteractionRequest(InteractionType::PassphraseState))
			}
		}
	}

	/// Get the passphrase request object or an error if not `PassphraseRequest`.
	pub fn passphrase_request(self) -> Result<PassphraseRequest<'a, T, R>> {
		match self {
			TrezorResponse::PassphraseRequest(r) => Ok(r),
			TrezorResponse::Ok(_) => Err(Error::UnexpectedMessageType(R::message_type())),
			TrezorResponse::Failure(m) => Err(Error::FailureResponse(m)),
			TrezorResponse::ButtonRequest(_) => {
				Err(Error::UnexpectedInteractionRequest(InteractionType::Button))
			}
			TrezorResponse::PinMatrixRequest(_) => {
				Err(Error::UnexpectedInteractionRequest(InteractionType::PinMatrix))
			}
			TrezorResponse::PassphraseStateRequest(_) => {
				Err(Error::UnexpectedInteractionRequest(InteractionType::PassphraseState))
			}
		}
	}

	/// Get the passphrase request object or an error if not `PassphraseStateRequest`.
	pub fn passphrase_state_request(self) -> Result<PassphraseStateRequest<'a, T, R>> {
		match self {
			TrezorResponse::PassphraseStateRequest(r) => Ok(r),
			TrezorResponse::Ok(_) => Err(Error::UnexpectedMessageType(R::message_type())),
			TrezorResponse::Failure(m) => Err(Error::FailureResponse(m)),
			TrezorResponse::ButtonRequest(_) => {
				Err(Error::UnexpectedInteractionRequest(InteractionType::Button))
			}
			TrezorResponse::PinMatrixRequest(_) => {
				Err(Error::UnexpectedInteractionRequest(InteractionType::PinMatrix))
			}
			TrezorResponse::PassphraseRequest(_) => {
				Err(Error::UnexpectedInteractionRequest(InteractionType::Passphrase))
			}
		}
	}
}

/// When resetting the device, it will ask for entropy to aid key generation.
pub struct EntropyRequest<'a> {
	client: &'a mut Trezor,
}

impl<'a> EntropyRequest<'a> {
	/// Provide exactly 32 bytes or entropy.
	pub fn ack_entropy(self, entropy: Vec<u8>) -> Result<TrezorResponse<'a, (), protos::Success>> {
		if entropy.len() != 32 {
			return Err(Error::InvalidEntropy);
		}

		let mut req = protos::EntropyAck::new();
		req.set_entropy(entropy);
		self.client.call(req, Box::new(|_, _| Ok(())))
	}
}

/// A Trezor client.
pub struct Trezor {
	model: TrezorModel,
	// Cached features for later inspection.
	features: Option<protos::Features>,
	transport: Box<Transport>,
}

/// Create a new Trezor instance with the given transport.
pub fn trezor_with_transport(model: TrezorModel, transport: Box<Transport>) -> Trezor {
	Trezor {
		model,
		transport: transport,
		features: None,
	}
}

impl Trezor {
	/// Get the model of the Trezor device.
	pub fn model(&self) -> TrezorModel {
		self.model
	}

	/// Get the features of the Trezor device.
	pub fn features(&self) -> Option<&protos::Features> {
		self.features.as_ref()
	}

	/// Sends a message and returns the raw ProtoMessage struct that was responded by the device.
	/// This method is only exported for users that want to expand the features of this library
	/// f.e. for supporting additional coins etc.
	pub fn call_raw<S: TrezorMessage>(&mut self, message: S) -> Result<ProtoMessage> {
		let proto_msg = ProtoMessage(S::message_type(), message.write_to_bytes()?);
		self.transport.write_message(proto_msg).map_err(|e| Error::TransportSendMessage(e))?;
		self.transport.read_message().map_err(|e| Error::TransportReceiveMessage(e))
	}

	/// Sends a message and returns a TrezorResponse with either the expected response message,
	/// a failure or an interaction request.
	/// This method is only exported for users that want to expand the features of this library
	/// f.e. for supporting additional coins etc.
	pub fn call<'a, T, S: TrezorMessage, R: TrezorMessage>(
		&'a mut self,
		message: S,
		result_handler: Box<ResultHandler<'a, T, R>>,
	) -> Result<TrezorResponse<'a, T, R>> {
		// trace!("Sending {:?} msg: {:?}", S::message_type(), message);
		let resp = self.call_raw(message)?;
		if resp.message_type() == R::message_type() {
			let resp_msg = resp.into_message()?;
			// trace!("Received {:?} msg: {:?}", R::message_type(), resp_msg);
			Ok(TrezorResponse::Ok(result_handler(self, resp_msg)?))
		} else {
			match resp.message_type() {
				MessageType_Failure => {
					let fail_msg = resp.into_message()?;
					// debug!("Received failure: {:?}", fail_msg);
					Ok(TrezorResponse::Failure(fail_msg))
				}
				MessageType_ButtonRequest => {
					let req_msg = resp.into_message()?;
					// trace!("Received ButtonRequest: {:?}", req_msg);
					Ok(TrezorResponse::ButtonRequest(ButtonRequest {
						message: req_msg,
						client: self,
						result_handler: result_handler,
					}))
				}
				MessageType_PinMatrixRequest => {
					let req_msg = resp.into_message()?;
					// trace!("Received PinMatrixRequest: {:?}", req_msg);
					Ok(TrezorResponse::PinMatrixRequest(PinMatrixRequest {
						message: req_msg,
						client: self,
						result_handler: result_handler,
					}))
				}
				MessageType_PassphraseRequest => {
					let req_msg = resp.into_message()?;
					// trace!("Received PassphraseRequest: {:?}", req_msg);
					Ok(TrezorResponse::PassphraseRequest(PassphraseRequest {
						message: req_msg,
						client: self,
						result_handler: result_handler,
					}))
				}
				MessageType_PassphraseStateRequest => {
					let req_msg = resp.into_message()?;
					// trace!("Received PassphraseStateRequest: {:?}", req_msg);
					Ok(TrezorResponse::PassphraseStateRequest(PassphraseStateRequest {
						message: req_msg,
						client: self,
						result_handler: result_handler,
					}))
				}
				mtype => {
					// debug!(
					// 	"Received unexpected msg type: {:?}; raw msg: {}",
					// 	mtype,
					// 	hex::encode(resp.into_payload())
					// );
					Err(Error::UnexpectedMessageType(mtype))
				}
			}
		}
	}

	pub fn init_device(&mut self) -> Result<()> {
		let features = self.initialize()?.ok()?;
		self.features = Some(features);
		Ok(())
	}

	pub fn initialize(&mut self) -> Result<TrezorResponse<Features, Features>> {
		let mut req = protos::Initialize::new();
		req.set_state(Vec::new());
		self.call(req, Box::new(|_, m| Ok(m)))
	}

	pub fn ping(&mut self, message: &str) -> Result<TrezorResponse<(), protos::Success>> {
		let mut req = protos::Ping::new();
		req.set_message(message.to_owned());
		self.call(req, Box::new(|_, _| Ok(())))
	}

    pub fn get_address(
        &mut self,
        path: Vec<u32>,
    ) -> Result<TrezorResponse<TezosAddress, TezosAddress>> {
        let mut req = protos::TezosGetAddress::new();
        req.set_address_n(path);

        self.call(req, Box::new(|_, m| Ok(m)))
    }
}
