use console::style;
use dialoguer::theme::ColorfulTheme;

use crate::common::exit_with_error;

#[derive(PartialEq, Debug, Clone)]
pub enum YesNoCustomAmount {
    Yes,
    No,
    Custom(u64),
}

impl YesNoCustomAmount {
    fn to_short_str(&self) -> String {
        match self {
            Self::Yes => "Y".to_string(),
            Self::No => "N".to_string(),
            Self::Custom(amount) => amount.to_string(),
        }
    }
}

pub fn yes_no_custom_amount_input(
    prompt: String,
    default: YesNoCustomAmount,

) -> YesNoCustomAmount
{
    let input = dialoguer::Input::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "{} {}",
            prompt,
            style("[y/n/(custom in µꜩ )]").bold(),
        ))
        .with_initial_text(default.to_short_str())
        .validate_with(|input: &String| -> Result<(), &str> {
            let input = input.trim().to_lowercase();
            if ["y", "n", "yes", "no"].contains(&input.as_str()) {
                return Ok(());
            }

            match input.parse::<u64>() {
                Ok(_) => Ok(()),
                Err(_) => {
                    if input.parse::<f32>().is_ok() {
                        Err("custom fee needs to be whole number in µꜩ  ( = 0.000 001 ꜩ ), but received a fraction.")
                    } else {
                        Err("inputted custom fee is not a valid number.")
                    }
                }
            }
        })
        .interact_text();

    let input = match input {
        Ok(input) => input.trim().to_lowercase(),
        Err(err) => exit_with_error(format!("invalid input. {}", err)),
    };

    if input.starts_with("y") {
        YesNoCustomAmount::Yes
    } else if input.starts_with("n") {
        YesNoCustomAmount::No
    } else {
        // unwrap is fine since validation above won't succeed unless
        // parsing of input is successful.
        YesNoCustomAmount::Custom(input.parse().unwrap())
    }
}
