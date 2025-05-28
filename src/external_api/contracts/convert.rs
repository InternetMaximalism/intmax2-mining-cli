use alloy::primitives::{Address as AlloyAddress, TxHash, B256, U256 as AlloyU256};
use intmax2_zkp::ethereum_types::{
    address::Address as IntmaxAddress, bytes32::Bytes32, u256::U256 as IntmaxU256,
    u32limb_trait::U32LimbTrait,
};

pub fn convert_u256_to_alloy(input: IntmaxU256) -> AlloyU256 {
    AlloyU256::from_be_slice(&input.to_bytes_be())
}

pub fn convert_u256_to_intmax(input: AlloyU256) -> IntmaxU256 {
    IntmaxU256::from_bytes_be(&input.to_be_bytes_vec())
}

pub fn convert_address_to_alloy(input: IntmaxAddress) -> AlloyAddress {
    AlloyAddress::from_slice(&input.to_bytes_be())
}

pub fn convert_address_to_intmax(input: AlloyAddress) -> IntmaxAddress {
    IntmaxAddress::from_bytes_be(&input.0 .0)
}

pub fn convert_b256_to_bytes32(input: B256) -> Bytes32 {
    Bytes32::from_bytes_be(&input.0)
}

pub fn convert_tx_hash_to_bytes32(input: TxHash) -> Bytes32 {
    Bytes32::from_bytes_be(&input.0)
}

pub fn convert_bytes32_to_tx_hash(input: Bytes32) -> TxHash {
    TxHash::from_slice(&input.to_bytes_be())
}

pub fn convert_bytes32_to_b256(input: Bytes32) -> B256 {
    B256::from_slice(&input.to_bytes_be())
}
