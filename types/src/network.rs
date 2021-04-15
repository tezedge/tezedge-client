use std::str::FromStr;

pub enum Network {
    Main(String),
    Beta(String),
    Florence(String),
    Edo(String),
    Delphi(String),
    Unknown(String),
}

impl FromStr for Network {
    // TODO: should be replaced with `!` and on user side: `.into_ok()`
    // once https://github.com/rust-lang/rust/issues/35121 gets stabilized.
    type Err = ();

    /// Parse network chain name string and get `Network`.
    ///
    /// Will never return an `Err`, since if the chain is unknown,
    /// `Ok(Self::Unknown)` will be returned.
    fn from_str(network_chain: &str) -> Result<Self, Self::Err> {
        let chain = network_chain.to_string();

        Ok(if chain.starts_with("TEZOS_MAINNET") {
            Self::Main(chain)
        } else if chain.starts_with("TEZOS_BETANET") {
            Self::Beta(chain)
        } else if chain.starts_with("TEZOS_FLORENCE") {
            Self::Florence(chain)
        } else if chain.starts_with("TEZOS_EDO") {
            Self::Edo(chain)
        } else if chain.starts_with("TEZOS_DELPHI") {
            Self::Delphi(chain)
        } else {
            Self::Unknown(chain)
        })
    }
}

impl ToString for Network {
    fn to_string(&self) -> String {
        match self {
            Self::Main(s) => s.clone(),
            Self::Beta(s) => s.clone(),
            Self::Florence(s) => s.clone(),
            Self::Edo(s) => s.clone(),
            Self::Delphi(s) => s.clone(),
            Self::Unknown(s) => s.clone(),
        }
    }
}
