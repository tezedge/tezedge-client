use protobuf;

use crate::protos::MessageType::*;
use crate::protos::*;

///! In this module we implement the message_type() getter for all protobuf message types.

/// This trait extends the protobuf Message trait to also have a static getter for the message
/// type code.  This getter is implemented in this file for all the messages we use.
pub trait TrezorMessage: protobuf::Message {
	fn message_type() -> MessageType;
}

/// This macro provides the TrezorMessage trait for a protobuf message.
macro_rules! trezor_message_impl {
	($struct:ident, $mtype:expr) => {
		impl TrezorMessage for $struct {
			fn message_type() -> MessageType {
				$mtype
			}
		}
	};
}

trezor_message_impl!(Initialize, MessageType_Initialize);
trezor_message_impl!(Ping, MessageType_Ping);
trezor_message_impl!(Success, MessageType_Success);
trezor_message_impl!(Failure, MessageType_Failure);
trezor_message_impl!(ChangePin, MessageType_ChangePin);
trezor_message_impl!(WipeDevice, MessageType_WipeDevice);
trezor_message_impl!(GetEntropy, MessageType_GetEntropy);
trezor_message_impl!(Entropy, MessageType_Entropy);
trezor_message_impl!(LoadDevice, MessageType_LoadDevice);
trezor_message_impl!(ResetDevice, MessageType_ResetDevice);
trezor_message_impl!(Features, MessageType_Features);
trezor_message_impl!(PinMatrixRequest, MessageType_PinMatrixRequest);
trezor_message_impl!(PinMatrixAck, MessageType_PinMatrixAck);
trezor_message_impl!(Cancel, MessageType_Cancel);
trezor_message_impl!(EndSession, MessageType_EndSession);
trezor_message_impl!(ApplySettings, MessageType_ApplySettings);
trezor_message_impl!(ButtonRequest, MessageType_ButtonRequest);
trezor_message_impl!(ButtonAck, MessageType_ButtonAck);
trezor_message_impl!(ApplyFlags, MessageType_ApplyFlags);
trezor_message_impl!(BackupDevice, MessageType_BackupDevice);
trezor_message_impl!(EntropyRequest, MessageType_EntropyRequest);
trezor_message_impl!(EntropyAck, MessageType_EntropyAck);
trezor_message_impl!(RecoveryDevice, MessageType_RecoveryDevice);
trezor_message_impl!(WordRequest, MessageType_WordRequest);
trezor_message_impl!(WordAck, MessageType_WordAck);
trezor_message_impl!(GetFeatures, MessageType_GetFeatures);
trezor_message_impl!(SetU2FCounter, MessageType_SetU2FCounter);
// trezor_message_impl!(FirmwareErase, MessageType_FirmwareErase);
// trezor_message_impl!(FirmwareUpload, MessageType_FirmwareUpload);
// trezor_message_impl!(FirmwareRequest, MessageType_FirmwareRequest);
// trezor_message_impl!(SelfTest, MessageType_SelfTest);
trezor_message_impl!(TezosGetAddress, MessageType_TezosGetAddress);
trezor_message_impl!(TezosAddress, MessageType_TezosAddress);
trezor_message_impl!(TezosSignTx, MessageType_TezosSignTx);
trezor_message_impl!(TezosSignedTx, MessageType_TezosSignedTx);
trezor_message_impl!(TezosGetPublicKey, MessageType_TezosGetPublicKey);
trezor_message_impl!(TezosPublicKey, MessageType_TezosPublicKey);
