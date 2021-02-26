use crate::common::exit_with_error;

// TODO: better parsing
pub fn parse_derivation_path(path: &str) -> Vec<u32> {
    path
        .replace("m/", "")
        .split("/")
        .map(|num| {
            let mut num = num.to_string();
            let is_hardened = num.ends_with("'");

            if is_hardened {
                num.pop();
            }

            let num = match num.parse() {
                Ok(n) => n,
                Err(_) => exit_with_error("invalid path"),
            };
            if is_hardened {
                num + 2147483648
            } else {
                num
            }
        })
        .collect()
}
