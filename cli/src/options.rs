use structopt::StructOpt;

use crate::transfer::Transfer;

/// Cli options definition
#[derive(StructOpt, Debug, Clone)]
pub struct Options {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    #[structopt(short = "E", long)]
    pub endpoint: String,

    #[structopt(subcommand)]
    pub sub_command: SubCommand,
}

#[derive(StructOpt, Debug, Clone)]
pub enum SubCommand {
    Transfer(Transfer),
}
