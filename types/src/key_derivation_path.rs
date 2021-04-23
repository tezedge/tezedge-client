use std::{fmt, str::FromStr};

pub const HARDENED_PATH: u32 = 2147483648;

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

#[derive(PartialEq, Debug, Clone)]
pub struct KeyDerivationPath(Vec<u32>);

impl fmt::Display for KeyDerivationPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path_str = self.0.iter()
            .map(|num| {
                if *num >= HARDENED_PATH {
                    format!("{}'", num - HARDENED_PATH)
                } else {
                    format!("{}", num)
                }
            })
            .collect::<Vec<_>>()
            .join("/");

        write!(f, "m/{}", path_str)
    }
}

impl KeyDerivationPath {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn take(self) -> Vec<u32> {
        self.0
    }
}

impl AsRef<[u32]> for KeyDerivationPath {
    fn as_ref(&self) -> &[u32] {
        &self.0
    }
}

impl FromStr for KeyDerivationPath {
    type Err = ParseDerivationPathError;

    fn from_str(path: &str) -> Result<Self, Self::Err> {
        if !path.starts_with("m/") {
            return Err(ParseDerivationPathError::new(
                ParseDerivationPathErrorKind::BadPrefix,
                path.to_string(),
                0,
                path.chars().position(|c| c == '/').unwrap_or(path.len() - 1) + 1,
            ));
        }

        Ok(KeyDerivationPath(path
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
                        num + HARDENED_PATH
                    } else {
                        num
                    })
                    .map_err(|_| {
                        ParseDerivationPathError::new(
                            ParseDerivationPathErrorKind::BadNumber,
                            path.to_string(),
                            // TODO: replace with correct position
                            0,
                            path.len(),
                        )
                    })
            })
            .collect::<Result<_, _>>()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation_path_parsing() {
        let path = "m/44'/1729'/0'/0'";
        assert_eq!(
            KeyDerivationPath::from_str(path).unwrap().to_string(),
            path,
        )
    }
}
