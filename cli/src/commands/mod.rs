use structopt::StructOpt;

pub mod address;
pub mod transfer;

#[derive(StructOpt)]
pub enum Command {
    Address(address::Address),
    Transfer(transfer::Transfer),
}
