use std::io::{Read as _, Write as _};

use aes_gcm::{aead::Aead, NewAead as _};
use ethers::{
    signers::Signer,
    types::{Address, H256},
};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

use crate::external_api::contracts::utils::get_wallet;

const NONCE: &'static str = "intmaxmining";

fn private_data_path() -> &'static str {
    let network = std::env::var("NETWORK").unwrap_or_else(|_| "testnet".into());
    match network.as_str() {
        "testnet" => "data/private.testnet.bin",
        "localnet" => "data/private.localnet.bin",
        _ => panic!("Unsupported network"),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrivateData {
    pub deposit_key: H256,
    pub claim_key: H256,
    pub withdrawal_address: Address,
}

#[derive(Debug)]
pub struct Addresses {
    pub deposit_address: Address,
    pub claim_address: Address,
    pub withdrawal_address: Address,
}

impl PrivateData {
    pub fn new(
        deposit_key: &str,
        claim_key: &str,
        withdrawal_address: &str,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            deposit_key: deposit_key.parse()?,
            claim_key: claim_key.parse()?,
            withdrawal_address: withdrawal_address.parse()?,
        })
    }

    pub fn encrypt(&self, password: &str) -> anyhow::Result<Vec<u8>> {
        let password_hash = keccak256_hash(password);
        let key = aes_gcm::Key::from_slice(&password_hash);
        let cipher = aes_gcm::Aes256Gcm::new(key);
        let nonce = aes_gcm::Nonce::from_slice(NONCE.as_bytes());
        let private_data_str = serde_json::to_string(self)?;
        let ciphertext = cipher
            .encrypt(nonce, private_data_str.as_bytes())
            .map_err(|_| anyhow::anyhow!("Failed to encrypt private data"))?;
        Ok(ciphertext)
    }

    pub fn decrypt(password: &str, ciphertext: &[u8]) -> anyhow::Result<Self> {
        let password_hash = keccak256_hash(password);
        let key = aes_gcm::Key::from_slice(&password_hash);
        let cipher = aes_gcm::Aes256Gcm::new(key);
        let nonce = aes_gcm::Nonce::from_slice(NONCE.as_bytes());
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| anyhow::anyhow!("Failed to decrypt private data"))?;
        let private_data = serde_json::from_slice(&plaintext)?;
        Ok(private_data)
    }

    pub async fn to_addresses(&self) -> anyhow::Result<Addresses> {
        let deposit_address = get_wallet(self.deposit_key).await?.address();
        let claim_address = get_wallet(self.claim_key).await?.address();

        Ok(Addresses {
            deposit_address,
            claim_address,
            withdrawal_address: self.withdrawal_address,
        })
    }
}

fn keccak256_hash(input: &str) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    let hash: [u8; 32] = result.into();
    hash
}

pub fn load_encrypted_private_data() -> Option<Vec<u8>> {
    let mut file = match std::fs::File::open(private_data_path()) {
        Ok(file) => file,
        Err(_) => return None,
    };
    let mut ciphertext = Vec::new();
    match file.read_to_end(&mut ciphertext) {
        Ok(_) => Some(ciphertext),
        Err(_) => None,
    }
}

pub fn write_encrypted_private_data(input: &[u8]) -> anyhow::Result<()> {
    let mut file = std::fs::File::create(private_data_path())?;
    file.write_all(input)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::PrivateData;

    #[test]
    fn encryption_data() {
        let password_str = "password";
        let private_data = PrivateData::new(
            "0xdf57089febbacf7ba0bc227dafbffa9fc08a93fdc68e1e42411a14efcf23656e",
            "0xdf57089febbacf7ba0bc227dafbffa9fc08a93fdc68e1e42411a14efcf23656e",
            "0x8626f6940E2eb28930eFb4CeF49B2d1F2C9C1199",
        )
        .unwrap();
        let ciphertext = private_data.encrypt(password_str).unwrap();
        let data = PrivateData::decrypt(password_str, &ciphertext).unwrap();
        assert_eq!(private_data.deposit_key, data.deposit_key);
    }

    #[tokio::test]
    async fn test_to_addresses() {
        let private_data = PrivateData::new(
            "0xdf57089febbacf7ba0bc227dafbffa9fc08a93fdc68e1e42411a14efcf23656e",
            "0xdf57089febbacf7ba0bc227dafbffa9fc08a93fdc68e1e42411a14efcf23656e",
            "0x8626f6940E2eb28930eFb4CeF49B2d1F2C9C1199",
        )
        .unwrap();
        let addresses = private_data.to_addresses().await.unwrap();
        dbg!(addresses);
    }
}
