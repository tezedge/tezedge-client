use structopt::StructOpt;

pub mod get_address;
pub use get_address::GetAddress;

#[derive(StructOpt, Debug, Clone)]
pub enum Address {
    Get(GetAddress),
}

impl Address {
    pub fn execute(self) {
        match self {
            Address::Get(c) => c.execute(),
        }
    }
}
