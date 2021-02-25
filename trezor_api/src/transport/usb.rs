use std::fmt;
use std::time::Duration;
use rusb::UsbContext;

use crate::{TrezorModel, AvailableDevice};
use super::{Error, Transport, AvailableDeviceTransport, Link, Protocol, ProtocolV1, ProtoMessage};

pub const CONFIG_ID: u8 = 0;
pub const INTERFACE_DESCRIPTOR: u8 = 0;
pub const LIBUSB_CLASS_VENDOR_SPEC: u8 = 0xff;

pub const INTERFACE: u8 = 0;
pub const INTERFACE_DEBUG: u8 = 1;
pub const ENDPOINT: u8 = 1;
pub const ENDPOINT_DEBUG: u8 = 2;
pub const READ_ENDPOINT_MASK: u8 = 0x80;

/// The chunk size for the serial protocol.
const CHUNK_SIZE: usize = 64;

const READ_TIMEOUT_MS: u64 = 100000;
const WRITE_TIMEOUT_MS: u64 = 100000;

/// An available transport for connecting with a device.
#[derive(Debug)]
pub struct AvailableUsbTransport {
	pub bus: u8,
	pub address: u8,
}

impl fmt::Display for AvailableUsbTransport {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "USB ({}:{})", self.bus, self.address)
	}
}

impl From<rusb::Device<rusb::Context>> for AvailableUsbTransport {
    fn from(dev: rusb::Device<rusb::Context>) -> Self {
        Self {
            bus: dev.bus_number(),
            address: dev.address(),
        }
    }
}

/// An actual serial HID USB link to a device over which bytes can be sent.
pub struct UsbLink {
	libusb_context: &'static rusb::Context,
	handle: &'static mut rusb::DeviceHandle<rusb::Context>,
	endpoint: u8,
}

impl Drop for UsbLink {
	fn drop(&mut self) {
		// Re-box the two static references and manually drop them.
		drop(unsafe { Box::from_raw(self.handle) });
		let context_ptr = self.libusb_context as *const _ as *mut rusb::Context;
		drop(unsafe { Box::from_raw(context_ptr) });
	}
}

impl Link for UsbLink {
	fn write_chunk(&mut self, chunk: Vec<u8>) -> Result<(), Error> {
		debug_assert_eq!(CHUNK_SIZE, chunk.len());
		let timeout = Duration::from_millis(WRITE_TIMEOUT_MS);
		if let Err(e) = self.handle.write_interrupt(self.endpoint, &chunk, timeout) {
			return Err(e)?;
		}
		Ok(())
	}

	fn read_chunk(&mut self) -> Result<Vec<u8>, Error> {
		let mut chunk = vec![0; CHUNK_SIZE];
		let endpoint = READ_ENDPOINT_MASK | self.endpoint;
		let timeout = Duration::from_millis(READ_TIMEOUT_MS);

		let n = self.handle.read_interrupt(endpoint, &mut chunk, timeout)?;
		if n == CHUNK_SIZE {
			Ok(chunk)
		} else {
			Err(Error::DeviceReadTimeout)
		}
	}
}

/// An implementation of the Transport interface for USB devices.
pub struct UsbTransport {
	protocol: ProtocolV1<UsbLink>,
}

impl UsbTransport {
	pub fn find_devices() -> Result<Vec<AvailableDevice>, Error> {
		let usb_ctx = rusb::Context::new()?;

		let mut devices = Vec::new();
		for dev in usb_ctx.devices()?.iter() {
			let desc = dev.device_descriptor()?;

			let model = match TrezorModel::from_device_descriptor(&desc) {
				Some(m) => m,
				None => continue,
			};

			// Check something with interface class code like python-trezor does.
			let class_code = dev
				.config_descriptor(CONFIG_ID)?
				.interfaces()
				.find(|i| i.number() == INTERFACE)
				.ok_or(rusb::Error::Other)?
				.descriptors()
				.find(|d| d.setting_number() == INTERFACE_DESCRIPTOR)
				.ok_or(rusb::Error::Other)?
				.class_code();
			if class_code != LIBUSB_CLASS_VENDOR_SPEC {
				continue;
			}

			devices.push(AvailableDevice {
				model: model,
				debug: false,
				transport: AvailableDeviceTransport::Usb(AvailableUsbTransport {
					bus: dev.bus_number(),
					address: dev.address(),
				}),
			});
		}
		Ok(devices)
	}

	/// Connect to a device over the USB transport.
	pub fn connect(device: &AvailableDevice) -> Result<Box<dyn Transport>, Error> {
		let transport = match device.transport {
			AvailableDeviceTransport::Usb(ref t) => t,
			_ => panic!("passed wrong AvailableDevice in UsbTransport::connect"),
		};

		let interface = match device.debug {
			false => INTERFACE,
			true => INTERFACE_DEBUG,
		};

		// To circumvent a limitation from the libusb crate, we need to do some unsafe stuff to be
		// able to store the context and the device handle.  We will allocate them on the heap using
		// boxes, but leak them into static references. In the Drop method for the Transport, we
		// will release the memory manually.

		let context = rusb::Context::new()?;
		let context_ptr = Box::into_raw(Box::new(context));
		let context_ref = unsafe { &*context_ptr as &'static rusb::Context };
		// Go over the devices again to match the desired device.
		let handle = {
			let dev = context_ref
				.devices()?
				.iter()
				.find(|dev| dev.bus_number() == transport.bus && dev.address() == transport.address)
				.ok_or(Error::DeviceDisconnected)?;
			// Check if there is not another device connected on this bus.
			let dev_desc = dev.device_descriptor()?;
			if TrezorModel::from_device_descriptor(&dev_desc).as_ref() != Some(&device.model) {
				return Err(Error::DeviceDisconnected);
			}
			let mut handle = dev.open()?;
			handle.claim_interface(interface)?;
			handle
		};
		let handle_ptr = Box::into_raw(Box::new(handle));
		let handle_ref = unsafe { &mut *handle_ptr as &'static mut rusb::DeviceHandle<rusb::Context> };

		Ok(Box::new(UsbTransport {
			protocol: ProtocolV1 {
				link: UsbLink {
					libusb_context: context_ref,
					handle: handle_ref,
					endpoint: match device.debug {
						false => ENDPOINT,
						true => ENDPOINT_DEBUG,
					},
				},
			},
		}))
	}
}

impl super::Transport for UsbTransport {
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
