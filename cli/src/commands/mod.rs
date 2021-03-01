use structopt::StructOpt;

pub mod address;
pub mod transfer;
pub mod delegate;

#[derive(StructOpt)]
pub enum Command {
    Address(address::Address),
    Transfer(transfer::Transfer),
    Delegate(delegate::Delegate),
}
