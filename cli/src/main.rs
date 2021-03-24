use structopt::StructOpt;

mod emojies;
mod spinner;
mod trezor;

mod common;
use common::exit_with_error;

mod commands;
use commands::Command;


fn main() {
    let command = Command::from_args();

    let result = match command {
        Command::Address(c) => c.execute(),
        Command::Transfer(c) => c.execute(),
        Command::Delegate(c) => c.execute(),
    };

    match result {
        Ok(_) => {}
        Err(err) => exit_with_error(err),
    }
}
