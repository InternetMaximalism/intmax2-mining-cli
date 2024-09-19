use std::{fs, path::Path};

use intmax2_zkp::wrapper_config::plonky2_config::PoseidonBN128GoldilocksConfig;
use mining_circuit::withdrawal::simple_withraw_circuit::SimpleWithdrawalValue;
use plonky2::{field::goldilocks_field::GoldilocksField, plonk::proof::ProofWithPublicInputs};
use serde::{Deserialize, Serialize};

const WITHDRAWAL_TEMP_PATH: &str = "temp/withdrawal_temp.json";

type F = GoldilocksField;
type C = PoseidonBN128GoldilocksConfig;
const D: usize = 2;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum WithdrawalStep {
    Plonky2Prove,
    GnarkStart,
    GnarkGetProof,
    ContractCall,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawalStatus {
    pub next_step: WithdrawalStep,
    pub witness: SimpleWithdrawalValue,
    pub plonlky2_proof: Option<ProofWithPublicInputs<F, C, D>>,
    pub job_id: Option<String>,
    pub start_query_time: Option<u64>, // unix timestamp
    pub gnark_proof: Option<String>,
}

impl WithdrawalStatus {
    pub fn new() -> anyhow::Result<Self> {
        let file = std::fs::read(WITHDRAWAL_TEMP_PATH)?;
        let status: Self = serde_json::from_slice(&file)?;
        Ok(status)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let file = serde_json::to_vec_pretty(&self)?;
        let path = Path::new(WITHDRAWAL_TEMP_PATH);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, file)?;
        Ok(())
    }

    pub fn delete() -> anyhow::Result<()> {
        std::fs::remove_file(WITHDRAWAL_TEMP_PATH)?;
        Ok(())
    }
}
