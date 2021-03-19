use std::fmt;

#[derive(thiserror::Error, PartialEq, Debug)]
pub struct ParseDerivationPathError {
    kind: ParseDerivationPathErrorKind,
    /// Key derivation path string input.
    path: String,
    /// Start of the index for the position where the error occured in the `path` string.
    start_index: usize,
    /// End of the index for the position where the error occured in the `path` string.
    end_index: usize,
}

impl ParseDerivationPathError {
    fn new(
        kind: ParseDerivationPathErrorKind,
        path: String,
        start_index: usize,
        end_index: usize,
    ) -> Self {
        Self { kind, path, start_index, end_index }
    }
}

impl fmt::Display for ParseDerivationPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: custom error for each kind
        "Invalid key derivation path".fmt(f)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum ParseDerivationPathErrorKind {
    BadPrefix,
    BadNumber,
}

// TODO: better parsing
pub fn parse_derivation_path(path: &str) -> Result<Vec<u32>, ParseDerivationPathError> {
    if !path.starts_with("m/") {
        return Err(ParseDerivationPathError::new(
            ParseDerivationPathErrorKind::BadPrefix,
            path.to_string(),
            0,
            path.chars().position(|c| c == '/').unwrap_or(path.len() - 1) + 1,
        ));
    }

    path
        .replace("m/", "")
        .split("/")
        .enumerate()
        .map(|(index, part)| {
            let mut num_str = part.to_string();
            let is_hardened = num_str.ends_with("'");

            if is_hardened {
                // remove the tick(')
                num_str.pop();
            }

            num_str.parse::<u32>()
                .map(|num| if is_hardened {
                num + 2147483648
            } else {
                num
        })
                .map_err(|_| {
                    ParseDerivationPathError::new(
                        ParseDerivationPathErrorKind::BadNumber,
                        path.to_string(),
                        // TODO: replace with correct position
                        0,
                        0,
                    )
                })
        })
        .collect()
}
