use std::error::Error;
use structopt::StructOpt;
use console::style;

use lib::api::GetVersionInfo;
use lib::http_api::HttpApi;

mod emojies;
mod spinner;
mod trezor;
mod ledger;

mod common;
use common::exit_with_error;

mod commands;
use commands::Command;

fn handle_endpoint(endpoint: &str) -> Result<(), Box<dyn Error>> {
    let api = HttpApi::new(endpoint.to_string());
    let version = api.get_version_info()?;

    eprintln!("Network: {}", style(&version.network_version.chain_name).bold());

    if !version.is_mainnet() {
        eprintln!(
            "         {} This is {} a Mainnet.",
            style("[WARN]").yellow(),
            style("NOT").yellow(),
        );
    }
    eprintln!();

    Ok(())
}

fn main() {
    let command = Command::from_args();

    if let Some(endpoint) = command.get_endpoint() {
        if let Err(err) = handle_endpoint(endpoint) {
            exit_with_error(err)
        }
    }

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
