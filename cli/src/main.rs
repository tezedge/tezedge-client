use structopt::StructOpt;

mod emojies;
mod spinner;

mod transfer;
use transfer::Transfer;

#[derive(StructOpt, Debug, Clone)]
pub enum Command {
    Transfer(Transfer),
}

fn main() {
    let command = Command::from_args();

    match command {
        Command::Transfer(transfer) => transfer.execute(),
    }
}
