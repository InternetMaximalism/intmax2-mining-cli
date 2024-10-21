use ethers::types::Address;
use intmax2_zkp::wrapper_config::plonky2_config::PoseidonBN128GoldilocksConfig;

use log::info;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::proof::{Proof, ProofWithPublicInputs},
};
use serde::{Deserialize, Serialize};

use crate::utils::{
    config::Settings,
    retry::with_retry,
    time::{sleep_for, sleep_until},
};

use super::error::{IntmaxError, IntmaxErrorResponse};

type F = GoldilocksField;
type C = PoseidonBN128GoldilocksConfig;
const D: usize = 2;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GnarkStartProofInput {
    pub address: Address,
    pub proof: Proof<F, C, D>,
    pub public_inputs: Vec<F>,
}

impl GnarkStartProofInput {
    pub fn new(address: Address, proof: ProofWithPublicInputs<F, C, D>) -> Self {
        Self {
            address,
            proof: proof.proof,
            public_inputs: proof.public_inputs,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum GnarkStartProofResponse {
    Success(GnarkStartProofSucessResponse),
    Error(IntmaxErrorResponse),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GnarkStartProofSucessResponse {
    pub job_id: String,
    pub status: String,
    pub estimated_time: Option<u64>, // in milliseconds
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum GnarkGetProofResponse {
    Success(GnarkGetProofSucessResponse),
    Error(IntmaxErrorResponse),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GnarkGetProofSucessResponse {
    pub job_id: String,
    pub status: String,
    pub result: Option<GnarkProof>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GnarkProof {
    pub public_inputs: Option<Vec<String>>,
    pub proof: String,
}

pub async fn gnark_start_prove(
    base_url: &str,
    address: Address,
    plonky2_proof: ProofWithPublicInputs<F, C, D>,
) -> Result<GnarkStartProofSucessResponse, IntmaxError> {
    info!(
        "gnark_start_prove with args address: {}, pis: {:?}",
        address, plonky2_proof.public_inputs
    );
    let input = GnarkStartProofInput::new(address, plonky2_proof);
    let response = with_retry(|| async {
        reqwest::Client::new()
            .post(format!("{}/start-proof", base_url))
            .json(&input)
            .send()
            .await
    })
    .await
    .map_err(|_| IntmaxError::NetworkError("failed to request gnark server".to_string()))?;
    let output: GnarkStartProofResponse = response.json().await.map_err(|e| {
        IntmaxError::SerializeError(format!("failed to parse response: {}", e.to_string()))
    })?;
    match output {
        GnarkStartProofResponse::Success(success) => Ok(success),
        GnarkStartProofResponse::Error(error) => Err(IntmaxError::ServerError(error)),
    }
}

pub async fn gnark_get_proof(
    base_url: &str,
    job_id: &str,
) -> Result<GnarkGetProofSucessResponse, IntmaxError> {
    info!("gnark_get_proof with arg job_id: {}", job_id);
    let response = with_retry(|| async {
        reqwest::Client::new()
            .get(format!("{}/get-proof?jobId={}", base_url, job_id))
            .send()
            .await
    })
    .await
    .map_err(|_| IntmaxError::NetworkError("failed to request gnark server".to_string()))?;
    let output: GnarkGetProofResponse = response.json().await.map_err(|e| {
        IntmaxError::SerializeError(format!("failed to parse response: {}", e.to_string()))
    })?;
    match output {
        GnarkGetProofResponse::Success(success) => Ok(success),
        GnarkGetProofResponse::Error(error) => Err(IntmaxError::ServerError(error)),
    }
}

pub async fn fetch_gnark_proof(
    base_url: &str,
    job_id: &str,
    start_query_time: u64,
) -> Result<GnarkProof, IntmaxError> {
    info!("fetch_gnark_proof for job_id: {}", job_id);
    let cooldown = Settings::load()
        .unwrap()
        .api
        .gnark_get_proof_cooldown_in_sec;
    sleep_until(start_query_time);

    let max_tries = 4;
    let mut tries = 0;
    loop {
        let output = gnark_get_proof(base_url, job_id).await?;
        if output.status == "done" {
            return Ok(output.result.unwrap());
        } else if output.status == "error" {
            return Err(IntmaxError::InternalError(
                "gnark server returned error".to_string(),
            ));
        }
        tries += 1;
        if tries > max_tries {
            return Err(IntmaxError::InternalError(
                "gnark server did not return proof in time".to_string(),
            ));
        }
        sleep_for(cooldown);
    }
}
