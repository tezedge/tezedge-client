//! Trezor Api
//!
//! To find available Trezor devices, use [find_devices].
//!
//! To connect to one of the available device, you need to call
//! [AvailableDevice::connect] on it, Which will give you [Trezor]
//! instance.
//!
//! After you have a [Trezor] instance, you need to initialize it with
//! [Trezor::init_device] before you can interact with the device.
//!
//! # Example
//! ```no_run
//! # use trezor_api::find_devices;
//!
//! let mut devices = find_devices().unwrap();
//!
//! // take the first device out of devices.
//! let device = devices.remove(0);
//!
//! let mut trezor = device.connect().unwrap();
//! trezor.init_device().unwrap();
//!
//! // After this you can interact with Trezor device.
//! let address = trezor.get_address(
//!     &"m/44'/1729'/0'/0'".parse().unwrap(),
//! ).unwrap().ack_all().unwrap();
//! ```
//!
//! # Interacting with Trezor
//!
//! On every call to Trezor, you will receive a [TrezorResponse]. As
//! you will notice based on type, you will receive:
//!
//! - [TrezorResponse::Ok(data)] meaning that command was successful
//!   and `data` will be whatever was requested from Trezor.
//! - [TrezorResponse::Failure] meaning there was an error when executing
//!   our command.
//! - We also might receive some action request, like [TrezorResponse::ButtonRequest]
//!   which basically tells us that the user needs to confirm the action
//!   on the device and we need to wait for it.
//!
//!   Then we need to [ButtonRequest::ack] that request, which will trigger
//!   Trezor to show a prompt on the device screen. `ack` method will block,
//!   untill user interacts with the device.
//!
//!   As a response to the `ack`, we might receive another request,
//!   failure or ok message.
//!
//!   With this architecture, we can first send a message to Trezor
//!   and after receiving response, if we get action request, before doing `ack`
//!   we can show the user on cli that he needs to confirm an action on the
//!   device. So that user won't have to guess why the cli is frozen and
//!   what it is waiting for.

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

/// A  Trezor device found by the `find_devices()` method.  It can be
/// connected to using the `connect()` method.
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
    /// Enable debug mode.
    ///
    /// Warning: it might cause Trezor client to not work.
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

/// Find Trezor devices.
pub fn find_devices() -> Result<Vec<AvailableDevice>> {
    let mut devices = Vec::new();
    use transport::usb::UsbTransport;
    devices.extend(UsbTransport::find_devices().map_err(|e| Error::TransportConnect(e))?);
    Ok(devices)
}
