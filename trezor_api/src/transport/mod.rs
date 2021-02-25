use std::fmt;

use crate::AvailableDevice;
use crate::protos::MessageType;

mod error;
pub use error::*;

mod protocol;
pub use protocol::*;

pub mod usb;
use usb::*;

pub mod udp;
use udp::*;

pub const DEV_TREZOR_ONE: (u16, u16) = (0x534C, 0x0001);
pub const DEV_TREZOR_T: (u16, u16) = (0x1209, 0x53C1);
// pub const DEV_TREZOR2_BL: (u16, u16) = (0x1209, 0x53C0);

/// An available transport for a Trezor device, containing any of the
/// different supported transports.
#[derive(Debug)]
pub enum AvailableDeviceTransport {
	// Hid(hid::AvailableHidTransport),
	Usb(AvailableUsbTransport),
	Udp(AvailableUdpTransport),
}

impl fmt::Display for AvailableDeviceTransport {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Usb(ref t) => write!(f, "{}", t),
			Self::Udp(ref t) => write!(f, "{}", t),
		}
	}
}

/// A protobuf message accompanied by the message type.  This type is used to pass messages over the
/// transport and used to contain messages received from the transport.
pub struct ProtoMessage(pub MessageType, pub Vec<u8>);

impl ProtoMessage {
	pub fn new(mt: MessageType, payload: Vec<u8>) -> ProtoMessage {
		ProtoMessage(mt, payload)
	}
	pub fn message_type(&self) -> MessageType {
		self.0
	}
	pub fn payload(&self) -> &[u8] {
		&self.1
	}
	pub fn into_payload(self) -> Vec<u8> {
		self.1
	}

	/// Take the payload from the ProtoMessage and parse it to a protobuf message.
	pub fn into_message<M: protobuf::Message>(self) -> Result<M, protobuf::error::ProtobufError> {
		Ok(protobuf::Message::parse_from_bytes(&self.into_payload())?)
	}
}

/// The transport interface that is implemented by the different ways to communicate with a Trezor
/// device.
pub trait Transport {
	fn session_begin(&mut self) -> Result<(), error::Error>;
	fn session_end(&mut self) -> Result<(), error::Error>;

	fn write_message(&mut self, message: ProtoMessage) -> Result<(), error::Error>;
	fn read_message(&mut self) -> Result<ProtoMessage, error::Error>;
}


pub fn connect(device: &AvailableDevice) -> Result<Box<dyn Transport>, Error> {
    match &device.transport {
        AvailableDeviceTransport::Usb(_) => UsbTransport::connect(device),
        AvailableDeviceTransport::Udp(_) => UdpTransport::connect(device),
    }
}
