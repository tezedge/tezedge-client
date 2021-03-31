use std::fmt;

pub mod protos;
pub mod messages;

mod error;
pub use error::*;

pub mod transport;
use transport::AvailableDeviceTransport;

mod client;
pub use client::*;

pub const DEV_TREZOR_ONE: (u16, u16) = (0x534C, 0x0001);
pub const DEV_TREZOR_T: (u16, u16) = (0x1209, 0x53C1);
// pub const DEV_TREZOR_T_BL: (u16, u16) = (0x1209, 0x53C0);
pub const CONFIG_ID: u8 = 0;
pub const INTERFACE_DESCRIPTOR: u8 = 0;
pub const LIBUSB_CLASS_VENDOR_SPEC: u8 = 0xff;

pub const INTERFACE: u8 = 0;
pub const INTERFACE_DEBUG: u8 = 1;
pub const ENDPOINT: u8 = 1;
pub const ENDPOINT_DEBUG: u8 = 2;
pub const READ_ENDPOINT_MASK: u8 = 0x80;

/// A device found by the `find_devices()` method.  It can be connected to using the `connect()`
/// method.
#[derive(Debug)]
pub struct AvailableDevice {
	pub model: TrezorModel,
	pub debug: bool,
	pub transport: AvailableDeviceTransport,
}

impl fmt::Display for AvailableDevice {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{} (transport: {}) (debug: {})", self.model, &self.transport, self.debug)
	}
}

impl AvailableDevice {
    pub fn set_debug_mode(&mut self) {
        self.debug = true;
    }

	/// Connect to the device.
	pub fn connect(self) -> Result<Trezor> {
        let t = transport::connect(&self).map_err(|e| Error::TransportConnect(e))?;
		Ok(client::trezor_with_transport(self.model, t))
	}
}



#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TrezorModel {
    /// Trezor Model One
    One,
    /// Trezor Model T
    T,
}

impl TrezorModel {
    pub fn from_device_descriptor(desc: &rusb::DeviceDescriptor) -> Option<Self> {
        match (desc.vendor_id(), desc.product_id()) {
            DEV_TREZOR_ONE => Some(TrezorModel::One),
            DEV_TREZOR_T => Some(TrezorModel::T),
            _ => None,
        }
    }
}

impl fmt::Display for TrezorModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let model_name = match self {
            TrezorModel::One => "One",
            TrezorModel::T => "T",
        };
        write!(f, "Trezor Model {}", model_name)
    }
}

pub fn find_devices() -> Result<Vec<AvailableDevice>> {
    let mut devices = Vec::new();
    use transport::usb::UsbTransport;
    devices.extend(UsbTransport::find_devices().map_err(|e| Error::TransportConnect(e))?);
    Ok(devices)
}
