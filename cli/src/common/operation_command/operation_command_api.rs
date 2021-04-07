use lib::api::*;

pub trait OperationCommandApi:
    GetChainID
    + GetHeadBlockHash
    + GetVersionInfo
    + GetProtocolInfo
    + GetContractCounter
    + GetContractManagerAddress
    + GetManagerPublicKey
    + GetPendingOperations
    + GetPendingOperationStatus
    + RunOperation
    + PreapplyOperations
    + InjectOperations
{}

impl<T> OperationCommandApi for T where T:
    GetChainID
    + GetHeadBlockHash
    + GetVersionInfo
    + GetProtocolInfo
    + GetContractCounter
    + GetContractManagerAddress
    + GetManagerPublicKey
    + GetPendingOperations
    + GetPendingOperationStatus
    + RunOperation
    + PreapplyOperations
    + InjectOperations
{}
