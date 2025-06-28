use serde::{Deserialize, Serialize};
use std::{env, fmt::Display, str::FromStr};
use strum_macros::EnumIter;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, EnumIter, Default)]
pub enum Network {
    Localnet,
    #[default]
    Base,
    Mainnet,
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::Localnet => write!(f, "localnet"),
            Network::Base => write!(f, "base"),
            Network::Mainnet => write!(f, "mainnet"),
        }
    }
}

impl FromStr for Network {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "localnet" => Ok(Network::Localnet),
            "mainnet" => Ok(Network::Mainnet),
            "base" => Ok(Network::Base),
            _ => Err(()),
        }
    }
}

pub fn get_network() -> Network {
    let network = env::var("NETWORK").unwrap_or_else(|_| Network::default().to_string());
    Network::from_str(&network).expect("Invalid network")
}

pub fn is_legacy() -> bool {
    get_network() == Network::Mainnet
}
