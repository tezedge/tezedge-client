use structopt::StructOpt;

mod options;
use options::{Options, SubCommand};

mod transfer;

fn main() {
    let opts = Options::from_args();

    match opts.sub_command.clone() {
        SubCommand::Transfer(transfer) => transfer.execute(opts),
    }
}
