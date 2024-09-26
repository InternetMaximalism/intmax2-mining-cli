use std::{env, fmt::Display};

#[derive(PartialEq)]
pub enum Network {
    Localnet,
    Sepolia,
    Mainnet,
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::Localnet => write!(f, "Localnet"),
            Network::Sepolia => write!(f, "Sepolia"),
            Network::Mainnet => write!(f, "Mainnet"),
        }
    }
}

pub fn get_network() -> Network {
    let network = env::var("NETWORK").unwrap_or_else(|_| "sepolia".to_string());
    match network.as_str() {
        "localnet" => Network::Localnet,
        "sepolia" => Network::Sepolia,
        "mainnet" => Network::Mainnet,
        _ => panic!("Invalid NETWORK environment variable"),
    }
}
