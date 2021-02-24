//! # Error Handling

/// Trezor error.
#[derive(Debug)]
pub enum Error {
	/// Error from libusb.
	Usb(rusb::Error),
    /// Error from Udp
    Udp(std::io::Error),
	/// The device to connect to was not found.
	DeviceNotFound,
	/// The device is no longer available.
	DeviceDisconnected,
	/// The HID version supported by the device was unknown.
	UnknownHidVersion,
	/// The device produced a data chunk of unexpected size.
	UnexpectedChunkSizeFromDevice(usize),
	/// Timeout expired while reading from device.
	DeviceReadTimeout,
	/// The device sent a chunk with a wrong magic value.
	DeviceBadMagic,
	/// The device sent a message with a wrong session id.
	DeviceBadSessionId,
	/// The device sent an unexpected sequence number.
	DeviceUnexpectedSequenceNumber,
	/// Received a non-existing message type from the device.
	InvalidMessageType(u32),
	/// Unable to determine device serial number.
	NoDeviceSerial,
}

impl From<rusb::Error> for Error {
	fn from(e: rusb::Error) -> Error {
		Error::Usb(e)
	}
}

