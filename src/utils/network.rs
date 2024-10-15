use std::{env, fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, EnumIter)]
pub enum Network {
    Localnet,
    Sepolia,
    Holesky,
    BaseSepolia,
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
            Network::BaseSepolia => write!(f, "base-sepolia"),
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
            "base-sepolia" => Ok(Network::BaseSepolia),
            _ => Err(()),
        }
    }
}

pub fn get_network() -> Network {
    let network = env::var("NETWORK").unwrap_or_else(|_| Network::default().to_string());
    Network::from_str(&network).expect("Invalid network")
}

pub fn is_legacy() -> bool {
    get_network() == Network::Mainnet || get_network() == Network::Holesky
}

// todo: change this to adapt to the Base network
pub fn is_testnet() -> bool {
    get_network() != Network::Mainnet
}

#[cfg(test)]
mod tests {}
