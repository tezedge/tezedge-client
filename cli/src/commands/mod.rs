use structopt::StructOpt;

pub mod address;
pub mod transfer;
pub mod delegate;

// These two are temporary, before local store will be implemented.
pub mod transfer_local;
pub mod delegate_local;

pub type CommandError = Box<dyn std::error::Error>;

#[derive(StructOpt)]
pub enum Command {
    Address(address::Address),
    Transfer(transfer::Transfer),
    Delegate(delegate::Delegate),
    TransferLocal(transfer_local::TransferLocal),
    DelegateLocal(delegate_local::DelegateLocal),
}

impl Command {
    /// Get node endpoint.
    pub fn get_endpoint(&self) -> Option<&str> {
        match self {
            Self::Address(_) => None,
            Self::Transfer(cmd) => Some(cmd.endpoint.as_str()),
            Self::Delegate(cmd) => Some(cmd.endpoint.as_str()),
            Self::TransferLocal(cmd) => Some(cmd.endpoint.as_str()),
            Self::DelegateLocal(cmd) => Some(cmd.endpoint.as_str()),
        }
    }
}
