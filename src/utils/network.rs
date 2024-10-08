use std::{env, fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, EnumIter)]
pub enum Network {
    Localnet,
    Sepolia,
    Holesky,
    Mainnet,
}

impl Default for Network {
    fn default() -> Self {
        Network::Holesky
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::Localnet => write!(f, "localnet"),
            Network::Sepolia => write!(f, "sepolia"),
            Network::Holesky => write!(f, "holesky"),
            Network::Mainnet => write!(f, "mainnet"),
        }
    }
}

impl FromStr for Network {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "localnet" => Ok(Network::Localnet),
            "sepolia" => Ok(Network::Sepolia),
            "holesky" => Ok(Network::Holesky),
            "mainnet" => Ok(Network::Mainnet),
            _ => Err(()),
        }
    }
}

pub fn get_network() -> Network {
    let network = env::var("NETWORK").unwrap_or_else(|_| Network::default().to_string());
    Network::from_str(&network).expect("Invalid network")
}

pub fn is_testnet() -> bool {
    get_network() != Network::Mainnet
}

#[cfg(test)]
mod tests {}
