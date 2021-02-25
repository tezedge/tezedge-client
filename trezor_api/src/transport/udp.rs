use std::fmt;
use std::time::Duration;
use std::net::UdpSocket;

use crate::{TrezorModel, AvailableDevice};
use super::{Error, Transport, AvailableDeviceTransport, Link, Protocol, ProtocolV1, ProtoMessage};

/// The chunk size for the serial protocol.
const CHUNK_SIZE: usize = 64;

const READ_TIMEOUT_MS: u64 = 100000;
const WRITE_TIMEOUT_MS: u64 = 100000;

/// An available transport for connecting with a device.
#[derive(Debug)]
pub struct AvailableUdpTransport {
	pub host: String,
	pub port: usize,
}

impl AvailableUdpTransport {
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl fmt::Display for AvailableUdpTransport {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "UDP ({})", self.address())
	}
}

pub struct UdpLink {
    socket: UdpSocket,
    address: String,
}

impl Link for UdpLink {
	fn write_chunk(&mut self, chunk: Vec<u8>) -> Result<(), Error> {
		debug_assert_eq!(CHUNK_SIZE, chunk.len());
        self.socket.send_to(&chunk, &self.address)
            .map_err(|err| Error::Udp(err))?;
		Ok(())
	}

	fn read_chunk(&mut self) -> Result<Vec<u8>, Error> {
		let mut chunk = vec![0; CHUNK_SIZE];

        let n = self.socket.recv(&mut chunk)
            .map_err(|err| Error::Udp(err))?;

		if n == CHUNK_SIZE {
			Ok(chunk)
		} else {
			Err(Error::DeviceReadTimeout)
		}
	}
}

/// An implementation of the Transport interface for UDP devices.
pub struct UdpTransport {
	protocol: ProtocolV1<UdpLink>,
}

impl UdpTransport {
	/// Connect to a device over the UDP transport.
	pub fn connect(device: &AvailableDevice) -> Result<Box<dyn Transport>, Error> {
		let transport = match device.transport {
			AvailableDeviceTransport::Udp(ref t) => t,
			_ => panic!("passed wrong AvailableDevice in UdpTransport::connect"),
		};

        // TODO: change endpoint
        let socket = UdpSocket::bind("127.0.0.1:34259")
            .map_err(|err| Error::Udp(err))?;

        let read_timeout = Duration::from_millis(READ_TIMEOUT_MS);
        let write_timeout = Duration::from_millis(WRITE_TIMEOUT_MS);
        let _ = socket.set_read_timeout(Some(read_timeout));
        let _ = socket.set_write_timeout(Some(write_timeout));

		Ok(Box::new(UdpTransport {
			protocol: ProtocolV1 {
                link: UdpLink {
                    socket,
                    address: transport.address(),
                },
            },
		}))
	}
}

impl super::Transport for UdpTransport {
	fn session_begin(&mut self) -> Result<(), Error> {
		self.protocol.session_begin()
	}
	fn session_end(&mut self) -> Result<(), Error> {
		self.protocol.session_end()
	}

	fn write_message(&mut self, message: ProtoMessage) -> Result<(), Error> {
		self.protocol.write(message)
	}
	fn read_message(&mut self) -> Result<ProtoMessage, Error> {
		self.protocol.read()
	}
}
