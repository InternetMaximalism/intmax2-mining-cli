use ethers::{types::H256, utils::keccak256};
use intmax2_zkp::{
    common::salt::Salt,
    ethereum_types::{bytes32::Bytes32, u256::U256, u32limb_trait::U32LimbTrait},
};
use std::str::FromStr;

pub fn get_salt_from_private_key_nonce(private_key: H256, nonce: u64) -> Salt {
    let deposit_salt_prefix =
        H256::from_str("0xbf21c6520d666a4167f35c091393809e314f62a8e5cb1c166dd4dcac3abe53ad")
            .unwrap();
    let prefixed_private_key = vec![deposit_salt_prefix.as_bytes(), private_key.as_bytes()]
        .concat()
        .to_vec();
    let hashed_private_key = keccak256(keccak256(&prefixed_private_key));
    let hashed_private_key_with_nonce =
        vec![hashed_private_key.to_vec(), nonce.to_be_bytes().to_vec()]
            .concat()
            .to_vec();
    let salt_bytes = Bytes32::from_bytes_be(&keccak256(&hashed_private_key_with_nonce));
    Salt(salt_bytes.reduce_to_hash_out())
}

/// Get the public key from a private key
/// TODO: Make it compatible with intmax2's specs. 
pub fn get_pubkey_from_private_key(private_key: H256) -> U256 {
    let deposit_salt_prefix =
        H256::from_str("0xbf21c6520d666a4167f35c091393809e314f62a8e5cb1c166dd4dcac3abe53ad")
            .unwrap();
    let prefixed_private_key = vec![deposit_salt_prefix.as_bytes(), private_key.as_bytes()]
        .concat()
        .to_vec();
    let hashed_private_key = keccak256(keccak256(&prefixed_private_key));
    let pubkey = U256::from_bytes_be(&hashed_private_key);
    pubkey
}
