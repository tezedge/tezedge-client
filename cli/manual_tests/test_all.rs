use std::fmt::{self, Display};
use std::process::{Command, Stdio};
use std::error::Error;
use structopt::StructOpt;
use console::{style, Term};

use lib::{Address, PublicKey, ToBase58Check};
use lib::api::GetManagerPublicKey;
use lib::http_api::HttpApi;
use cli_spinner::SpinnerBuilder;

#[derive(thiserror::Error, Debug)]
enum BuildError {
    #[error("cargo build failed! IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("cargo build failed! Raw Output:\n{0}")]
    Output(String),
}

#[derive(thiserror::Error, Debug)]
struct CommandError {
    command: String,
    output: String,
}

impl Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "Cli command failed!\nCommand: {}\n\n Raw Output:\n\n{}",
            self.command,
            self.output,
        )
    }
}

/// Test cli functionalities.
///
/// Pass public and private keys of the account, which will be the
/// source of funds. Needs to have tezos in balance!
#[derive(StructOpt)]
pub struct TestAll {
    #[structopt(short = "E", long)]
    pub endpoint: String,

    #[structopt(long = "public-key")]
    pub public_key: String,

    #[structopt(long = "private-key")]
    pub private_key: String,

    // #[structopt(long = "trezor")]
    // pub use_trezor: bool,

    // #[structopt(long = "ledger")]
    // pub use_ledger: bool,

    /// Use --release mode when building: `cargo build --release`.
    #[structopt(long)]
    pub release: bool,
}

impl TestAll {
    fn api(&self) -> HttpApi {
        HttpApi::new(&self.endpoint)
    }

    fn cli_command(&self) -> Command {
        let target_dir = std::env::var("CARGO_TARGET_DIR")
            .unwrap_or_else(|_| String::from("target"));
        let cli_path = format!(
            "{}/{}/cli",
            target_dir,
            if self.release { "release" } else { "debug" },
        );

        let mut command = Command::new(cli_path);
        // enable colors
        command.env("CLICOLOR", "1");
        command.env("CLICOLOR_FORCE", "1");

        command
    }

    fn spinner_for_build(&self) -> SpinnerBuilder {
        let mut spinner = SpinnerBuilder::new()
            .with_text("cargo build");

        if self.release {
            spinner = spinner.with_text("cargo build --release")
        }

        spinner
    }

    fn build(&self) -> Result<(), BuildError> {
        let mut command = Command::new("cargo");
        command.arg("build");
        // enable colors
        command.arg("--color").arg("always");

        if self.release {
            command.arg("--release");
        }

        let output = command.output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(BuildError::Output(
                String::from_utf8_lossy(&output.stderr).to_string()
            ))
        }
    }

    fn get_address_command(&self, key_path: &str, device_flag: &str) -> Command {
        let mut command = self.cli_command();
        command
            .arg("address").arg("get")
            .arg(device_flag)
            .arg("--path").arg(key_path)
            .stderr(Stdio::inherit());
        command
    }

    fn get_address(&self, key_path: &str, device_flag: &str) -> Result<Address, Box<dyn Error>> {
        let mut command = self.get_address_command(key_path, device_flag);
        let command_str = format!("{:?}", &command);
        let output = command.output()?;

        if output.status.success() {
            let addr_str = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            Ok(Address::from_base58check(&addr_str)?)
        } else {
            Err(CommandError {
                command: command_str,
                output: String::from_utf8_lossy(&output.stderr).to_string(),
            }.into())
        }
    }

    fn get_address_trezor(&self, key_path: &str) -> Result<Address, Box<dyn Error>> {
        self.get_address(key_path, "--trezor")
    }

    fn get_address_ledger(&self, key_path: &str) -> Result<Address, Box<dyn Error>> {
        self.get_address(key_path, "--ledger")
    }

    fn transfer_local_command(
        &self,
        from: &Address,
        to: &Address,
        amount: &str,
        fee: Option<&str>,
    ) -> Command
    {
        let mut command = self.cli_command();
        command
            .arg("transfer-local")
            .arg("--no-prompt")
            .arg("--endpoint").arg(&self.endpoint)
            .arg("--private-key").arg(&self.private_key)
            .arg("--public-key").arg(&self.public_key)
            .arg("--from").arg(from.to_base58check())
            .arg("--to").arg(to.to_base58check())
            .arg("--amount").arg(amount);

        if let Some(fee) = fee {
            command.arg("--fee").arg(fee);
        }

        command
    }

    fn transfer_local(
        &self,
        from: &Address,
        to: &Address,
        amount: &str,
        fee: Option<&str>,
    ) -> Result<String, CommandError>
    {
        let mut command = self.transfer_local_command(from, to, amount, fee.clone());
        let command_str = format!("{:?}", &command);

        let output = command.output().expect("failed to get output of transfer-local command!");

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(CommandError {
                command: command_str,
                output: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
    }

    fn hw_transfer_command(
        &self,
        device_flag: &str,
        from: &str,
        to: &Address,
        amount: &str,
        fee: Option<&str>,
    ) -> Command
    {
        let mut command = self.cli_command();
        command
            .arg("transfer")
            .arg("--no-prompt")
            .arg(device_flag)
            .arg("--endpoint").arg(&self.endpoint)
            .arg("--from").arg(from)
            .arg("--to").arg(to.to_base58check())
            .arg("--amount").arg(amount);

        if let Some(fee) = fee {
            command.arg("--fee").arg(fee);
        }

        command
    }

    fn hw_transfer(
        &self,
        device_flag: &str,
        from: &str,
        to: &Address,
        amount: &str,
        fee: Option<&str>,
    ) -> Result<String, CommandError>
    {
        let mut command = self.hw_transfer_command(
            device_flag, from, to, amount, fee.clone(),
        );
        let command_str = format!("{:?}", &command);

        let output = command.output().expect("failed to get output of transfer command!");

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(CommandError {
                command: command_str,
                output: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
    }

    fn trezor_transfer(
        &self,
        from: &str,
        to: &Address,
        amount: &str,
        fee: Option<&str>,
    ) -> Result<String, CommandError>
    {
        self.hw_transfer("--trezor", from, to, amount, fee)
    }

    fn ledger_transfer(
        &self,
        from: &str,
        to: &Address,
        amount: &str,
        fee: Option<&str>,
    ) -> Result<String, CommandError>
    {
        self.hw_transfer("--ledger", from, to, amount, fee)
    }

    fn hw_find_unrevealed(
        &self,
        device: &str,
        prefix: &str,
        index: u32,
    ) -> Result<(u32, String, Address), Box<dyn Error>>
    {
        let rpc = self.api();
        let device_flag = format!("--{}", device);
        let mut i = index;

        loop {
            if i != index {
                let _ = Term::stderr().clear_last_lines(1);
            }
            let path = format!("{}/{}'", prefix, i);
            eprintln!(
                " -   searching for unrevealed {} account. Checking path: {}",
                style(device).bold(),
                style(&path).magenta(),
            );

            let address = self.get_address(&path, &device_flag)?;

            if rpc.get_manager_public_key(&address)?.is_none() {
                let _ = Term::stderr().clear_last_lines(1);
                eprintln!(
                    " {} {} unrevealed account ({}) found on path: {}",
                    style(emojies::TICK),
                    style(device).bold(),
                    style(address.to_base58check()).bold(),
                    style(&path).magenta(),
                );
                break Ok((i, path, address));
            }

            i += 1;
        }
    }

    fn trezor_find_unrevealed(
        &self,
        prefix: &str,
        index: u32,
    ) -> Result<(u32, String, Address), Box<dyn Error>>
    {
        self.hw_find_unrevealed("trezor", prefix, index)
    }

    fn ledger_find_unrevealed(
        &self,
        prefix: &str,
        index: u32,
    ) -> Result<(u32, String, Address), Box<dyn Error>>
    {
        self.hw_find_unrevealed("ledger", prefix, index)
    }

    /// Test transfer + reveal from hardware wallet.
    fn test_hw_transfer_reveal(
        &self,
        device: &str,
        key_path: &str,
        to: &Address,
    ) -> Result<(), Box<dyn Error>>
    {
        let device_flag = "--".to_string() + device;
        let mut spinner = SpinnerBuilder::new()
            .with_text(format!(
                "testing {} from {} account",
                style("transfer + reveal").yellow(),
                style(device).bold(),
            ))
            .start();

        let op_hash = spinner.fail_if(
            self.hw_transfer(&device_flag, key_path, to, "0.1", None),
        )?;
        spinner.finish_succeed(format!(
            "{} from {} account successful. Operation hash: {}",
            style("transfer + reveal").green(),
            style(device).bold(),
            style(op_hash).cyan(),
        ));

        Ok(())
    }

    /// Test transfer without reveal from hardware wallet.
    fn test_hw_transfer(
        &self,
        device: &str,
        key_path: &str,
        to: &Address,
    ) -> Result<(), Box<dyn Error>>
    {
        let device_flag = "--".to_string() + device;
        let mut spinner = SpinnerBuilder::new()
            .with_text(format!(
                "testing {} from {} account",
                style("transfer w/o reveal").yellow(),
                style(device).bold(),
            ))
            .start();

        let op_hash = spinner.fail_if(
            self.hw_transfer(&device_flag, key_path, to, "0.1", None),
        )?;
        spinner.finish_succeed(format!(
            "{} from {} account successful. Operation hash: {}",
            style("transfer w/o reveal").green(),
            style(device).bold(),
            style(op_hash).cyan(),
        ));

        Ok(())
    }

    pub fn test(self) -> Result<(), Box<dyn Error>> {
        let mut spinner = self.spinner_for_build().start();
        spinner.fail_if(self.build())?;
        spinner.finish_succeed("build successful!");

        let key_path_prefix = "m/44'/1729'/0'".to_string();
        let public_key = PublicKey::from_base58check(&self.public_key)
            .map_err(|_| "invalid --public-key passed")?;

        let local_address: Address = public_key.hash().into();
        let (i, trezor_key_path, trezor_address)
            = self.trezor_find_unrevealed(&key_path_prefix, 0)?;
        let (_, ledger_key_path, ledger_address)
            = self.ledger_find_unrevealed(&key_path_prefix, i)?;

        // Transfer to trezor account.
        let mut spinner = SpinnerBuilder::new()
            .with_text(format!(
                "transfering funds to trezor account: {}",
                style(trezor_address.to_base58check()).bold(),
            ))
            .start();

        let op_hash = spinner.fail_if(
            self.transfer_local(&local_address, &trezor_address, "2", None),
        )?;
        spinner.finish_succeed(format!(
            "funds transfered to trezor account. Operation hash: {}",
            style(op_hash).cyan(),
        ));

        // Transfer to ledger account.
        let mut spinner = SpinnerBuilder::new()
            .with_text(format!(
                "transfering funds to ledger account: {}",
                style(trezor_address.to_base58check()).bold(),
            ))
            .start();

        let op_hash = spinner.fail_if(
            self.transfer_local(&local_address, &ledger_address, "2", None),
        )?;
        spinner.finish_succeed(format!(
            "funds transfered to ledger account. Operation hash: {}",
            style(op_hash).cyan(),
        ));

        // test transfer + reveal
        self.test_hw_transfer_reveal("trezor", &trezor_key_path, &ledger_address)?;
        self.test_hw_transfer_reveal("ledger", &ledger_key_path, &trezor_address)?;

        // test transfer with already revealed accounts
        self.test_hw_transfer("trezor", &trezor_key_path, &ledger_address)?;
        self.test_hw_transfer("ledger", &ledger_key_path, &trezor_address)?;

        Ok(())
    }
}

fn main() {
    let _ = TestAll::from_args().test();
}