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
