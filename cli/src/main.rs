use structopt::StructOpt;

mod emojies;
mod spinner;

mod commands;
use commands::Command;


fn main() {
    let command = Command::from_args();

    match command {
        Command::Address(c) => c.execute(),
        Command::Transfer(c) => c.execute(),
    }
}
