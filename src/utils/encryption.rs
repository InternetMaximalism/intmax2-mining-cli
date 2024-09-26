use aes_gcm::{aead::Aead, NewAead as _};
use serde::Serialize;
use sha3::{Digest, Keccak256};

const NONCE: &'static str = "intmaxmining";

pub fn encrypt<T: Serialize>(password: &str, value: &T) -> anyhow::Result<Vec<u8>> {
    let password_hash = keccak256_hash(password);
    let key = aes_gcm::Key::from_slice(&password_hash);
    let cipher = aes_gcm::Aes256Gcm::new(key);
    let nonce = aes_gcm::Nonce::from_slice(NONCE.as_bytes());
    let private_data_str = serde_json::to_string(value)?;
    let ciphertext = cipher
        .encrypt(nonce, private_data_str.as_bytes())
        .map_err(|_| anyhow::anyhow!("Failed to encrypt private data"))?;
    Ok(ciphertext)
}

pub fn decrypt<T>(password: &str, ciphertext: &[u8]) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let password_hash = keccak256_hash(password);
    let key = aes_gcm::Key::from_slice(&password_hash);
    let cipher = aes_gcm::Aes256Gcm::new(key);
    let nonce = aes_gcm::Nonce::from_slice(NONCE.as_bytes());
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("Failed to decrypt private data"))?;
    let private_data: T = serde_json::from_slice(&plaintext)?;
    Ok(private_data)
}

fn keccak256_hash(input: &str) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    let hash: [u8; 32] = result.into();
    hash
}
