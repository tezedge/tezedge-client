use std::process;
use std::fmt::Display;
use console::style;

pub fn exit_with_error<E: Display>(error: E) -> ! {
    eprintln!(
        "{} {}",
        style("[ERROR]").red().bold(),
        error,
    );
    process::exit(1)
}
