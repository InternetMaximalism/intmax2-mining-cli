use std::time::UNIX_EPOCH;

use anyhow::ensure;
use contract::claim_tokens;
use mining_circuit::claim::claim_circuit::ClaimPublicInputs;

use crate::{
    cli::console::print_status,
    config::Settings,
    external_api::{
        contracts::events::Deposited,
        intmax::gnark::{fetch_gnark_proof, gnark_start_prove},
    },
    state::state::State,
};

use super::*;

pub async fn claim_task(state: &State, events: &[Deposited]) -> anyhow::Result<()> {
    from_step1(state, events).await?;
    Ok(())
}

pub async fn resume_claim_task(state: &State) -> anyhow::Result<()> {
    let status = match temp::ClaimStatus::new() {
        Ok(status) => status,
        Err(_) => return Ok(()),
    };
    match status.next_step {
        temp::ClaimStep::Plonky2Prove => from_step2(state).await?,
        temp::ClaimStep::GnarkStart => from_step3(state).await?,
        temp::ClaimStep::GnarkGetProof => from_step4(state).await?,
        temp::ClaimStep::ContractCall => from_step5(state).await?,
    }
    Ok(())
}

// Generate witness
async fn from_step1(state: &State, events: &[Deposited]) -> anyhow::Result<()> {
    print_status("Claim: generating claim witness");
    let witness = witness_generation::generate_claim_witness(state, events).await?;
    let status = temp::ClaimStatus {
        next_step: temp::ClaimStep::Plonky2Prove,
        witness: witness.clone(),
        plonlky2_proof: None,
        job_id: None,
        start_query_time: None,
        gnark_proof: None,
    };
    status.save()?;
    from_step2(state).await?;
    Ok(())
}

// Prove with Plonky2
async fn from_step2(state: &State) -> anyhow::Result<()> {
    print_status("Claim: proving with plonky2");
    let mut status = temp::ClaimStatus::new()?;
    ensure!(status.next_step == temp::ClaimStep::Plonky2Prove);
    ensure!(state.prover.is_some(), "Prover is not initialized");
    let mut cyclic_proof = None;
    for w in &status.witness {
        let proof = state
            .prover
            .as_ref()
            .unwrap()
            .claim_processor
            .prove(w, &cyclic_proof)?;
        cyclic_proof = Some(proof);
    }
    let plonky2_proof = state
        .prover
        .as_ref()
        .unwrap()
        .claim_wrapper_processor
        .prove(&cyclic_proof.unwrap())?;
    status.plonlky2_proof = Some(plonky2_proof.clone());
    status.next_step = temp::ClaimStep::GnarkStart;
    status.save()?;
    from_step3(state).await?;
    Ok(())
}

// Start Gnark
async fn from_step3(state: &State) -> anyhow::Result<()> {
    print_status("Claim: starting gnark prover");
    let mut status = temp::ClaimStatus::new()?;
    ensure!(status.next_step == temp::ClaimStep::GnarkStart);
    let settings = Settings::new()?;
    let withdrawal_address = state.private_data.withdrawal_address.clone();

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
    from_step4(state).await?;
    Ok(())
}

// Get Gnark proof
async fn from_step4(_state: &State) -> anyhow::Result<()> {
    print_status("Claim: getting gnark proof");
    let mut status = temp::ClaimStatus::new()?;
    ensure!(status.next_step == temp::ClaimStep::GnarkGetProof);
    let settings = Settings::new()?;
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
    from_step5(_state).await?;
    Ok(())
}

// Call contract
async fn from_step5(state: &State) -> anyhow::Result<()> {
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
    claim_tokens(
        state.private_data.claim_key,
        &claims,
        pis,
        &status.gnark_proof.unwrap(),
    )
    .await?;
    temp::ClaimStatus::delete()?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use determin::ClaimProcess;

    use crate::{state::prover::Prover, test::get_dummy_state};

    use super::*;

    #[tokio::test]
    async fn test_claim_task() {
        let mut state = get_dummy_state();
        state.sync_trees().await.unwrap();
        let process = determin::determin_next_claim_process(&state).await.unwrap();
        dbg!(&process);

        let prover = Prover::new();
        state.prover = Some(prover);

        let events = match process {
            ClaimProcess::Claim(events) => events,
            _ => panic!("Invalid process"),
        };
        claim_task(&state, &events).await.unwrap();
    }
}
