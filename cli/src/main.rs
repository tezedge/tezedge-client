use structopt::StructOpt;

mod emojies;
mod spinner;

mod commands;
use commands::Command;


fn main() {
    let command = Command::from_args();

    match command {
        Command::Transfer(transfer) => transfer.execute(),
    }
}
