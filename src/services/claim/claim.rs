use std::time::UNIX_EPOCH;

use anyhow::ensure;
use contract::claim_tokens;
use mining_circuit_v1::claim::claim_circuit::ClaimPublicInputs;

use crate::{
    cli::console::print_status,
    external_api::{
        contracts::events::Deposited,
        intmax::gnark::{fetch_gnark_proof, gnark_start_prove},
    },
    state::{key::Key, state::State},
    utils::config::Settings,
};

use super::*;

pub async fn single_claim_task(
    state: &mut State,
    key: &Key,
    is_short_term: bool,
    events: &[Deposited],
) -> anyhow::Result<()> {
    from_step1(state, key, is_short_term, events).await?;
    Ok(())
}

pub async fn resume_claim_task(state: &State, key: &Key) -> anyhow::Result<()> {
    let status = match temp::ClaimStatus::new() {
        Ok(status) => status,
        Err(_) => return Ok(()),
    };
    match status.next_step {
        temp::ClaimStep::Plonky2Prove => from_step2(state, key).await?,
        temp::ClaimStep::GnarkStart => from_step3(state, key).await?,
        temp::ClaimStep::GnarkGetProof => from_step4(state, key).await?,
        temp::ClaimStep::ContractCall => from_step5(state, key).await?,
    }
    Ok(())
}

// Generate witness
async fn from_step1(
    state: &State,
    key: &Key,
    is_short_term: bool,
    events: &[Deposited],
) -> anyhow::Result<()> {
    print_status("Claim: generating claim witness");
    let witness =
        witness_generation::generate_claim_witness(state, key, is_short_term, events).await?;
    let status = temp::ClaimStatus {
        is_short_term,
        next_step: temp::ClaimStep::Plonky2Prove,
        witness: witness.clone(),
        plonlky2_proof: None,
        job_id: None,
        start_query_time: None,
        gnark_proof: None,
    };
    status.save()?;
    from_step2(state, key).await?;
    Ok(())
}

// Prove with Plonky2
async fn from_step2(state: &State, key: &Key) -> anyhow::Result<()> {
    print_status("Claim: proving with plonky2");
    let mut status = temp::ClaimStatus::new()?;
    ensure!(status.next_step == temp::ClaimStep::Plonky2Prove);
    let mut cyclic_proof = None;
    for w in &status.witness {
        let proof = state.prover.claim_processor().prove(w, &cyclic_proof)?;
        cyclic_proof = Some(proof);
    }
    let plonky2_proof = state
        .prover
        .claim_wrapper_processor()
        .prove(&cyclic_proof.unwrap())?;
    status.plonlky2_proof = Some(plonky2_proof.clone());
    status.next_step = temp::ClaimStep::GnarkStart;
    status.save()?;
    from_step3(state, key).await?;
    Ok(())
}

// Start Gnark
async fn from_step3(state: &State, key: &Key) -> anyhow::Result<()> {
    print_status("Claim: starting gnark prover");
    let mut status = temp::ClaimStatus::new()?;
    ensure!(status.next_step == temp::ClaimStep::GnarkStart);
    let settings = Settings::load()?;
    let withdrawal_address = key.withdrawal_address;

    let prover_url = settings.api.claim_gnark_prover_url.clone();
    let plonky2_proof = status.plonlky2_proof.clone().unwrap();
    let output = gnark_start_prove(&prover_url, withdrawal_address, plonky2_proof).await?;
    status.job_id = Some(output.job_id.clone());
    let now: u64 = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    status.start_query_time = Some(output.estimated_time.unwrap_or(0) / 1000 + now);
    status.next_step = temp::ClaimStep::GnarkGetProof;
    status.save()?;
    from_step4(state, key).await?;
    Ok(())
}

// Get Gnark proof
async fn from_step4(state: &State, key: &Key) -> anyhow::Result<()> {
    print_status("Claim: getting gnark proof");
    let mut status = temp::ClaimStatus::new()?;
    ensure!(status.next_step == temp::ClaimStep::GnarkGetProof);
    let settings = Settings::load()?;
    let prover_url = settings.api.claim_gnark_prover_url.clone();
    let output = fetch_gnark_proof(
        &prover_url,
        status.job_id.as_ref().unwrap(),
        status.start_query_time.unwrap(),
    )
    .await?;
    status.gnark_proof = Some(output.proof.clone());
    status.next_step = temp::ClaimStep::ContractCall;
    status.save()?;
    from_step5(state, key).await?;
    Ok(())
}

// Call contract
async fn from_step5(_state: &State, key: &Key) -> anyhow::Result<()> {
    print_status("Claim: calling contract");
    let status = temp::ClaimStatus::new()?;
    ensure!(status.next_step == temp::ClaimStep::ContractCall);
    let mut claims = Vec::new();
    for w in &status.witness {
        claims.push(w.claim.clone());
    }
    let last_claim_hash = status.witness.last().unwrap().new_claim_hash;
    let pis = ClaimPublicInputs {
        deposit_tree_root: status.witness[0].deposit_tree_root,
        eligible_tree_root: status.witness[0].eligible_tree_root,
        last_claim_hash,
    };
    temp::ClaimStatus::delete()?;
    claim_tokens(
        key.withdrawal_private_key,
        status.is_short_term,
        &claims,
        pis,
        &status.gnark_proof.unwrap(),
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::test::{get_dummy_keys, get_dummy_state};

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_claim_task() {
        let dummy_key = get_dummy_keys();

        let mut state = get_dummy_state().await;
        let assets_status = state.sync_and_fetch_assets(&dummy_key).await.unwrap();

        let is_short_term = true;
        let not_claimed_events = assets_status.get_not_claimed_events(is_short_term);
        assert!(not_claimed_events.len() > 0);

        single_claim_task(&mut state, &dummy_key, is_short_term, &not_claimed_events)
            .await
            .unwrap();
    }
}
