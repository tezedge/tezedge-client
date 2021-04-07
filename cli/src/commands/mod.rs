use structopt::StructOpt;

pub mod address;
pub mod transfer;
pub mod delegate;

pub type CommandError = Box<dyn std::error::Error>;

#[derive(StructOpt)]
pub enum Command {
    Address(address::Address),
    Transfer(transfer::Transfer),
    Delegate(delegate::Delegate),
}

impl Command {
    /// Get node endpoint.
    pub fn get_endpoint(&self) -> Option<&str> {
        match self {
            Self::Address(_) => None,
            Self::Transfer(cmd) => Some(cmd.endpoint.as_str()),
            Self::Delegate(cmd) => Some(cmd.endpoint.as_str()),
        }
    }
}
