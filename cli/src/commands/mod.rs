use structopt::StructOpt;

pub mod transfer;

#[derive(StructOpt, Debug, Clone)]
pub enum Command {
    Transfer(transfer::Transfer),
}
