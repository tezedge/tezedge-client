use structopt::StructOpt;

pub mod address;
pub mod transfer;

#[derive(StructOpt, Debug, Clone)]
pub enum Command {
    Address(address::Address),
    Transfer(transfer::Transfer),
}
