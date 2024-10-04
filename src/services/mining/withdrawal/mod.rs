use std::time::UNIX_EPOCH;

use anyhow::ensure;
use mining_circuit_v1::withdrawal::simple_withraw_circuit::SimpleWithdrawalPublicInputs;

use crate::{
    cli::console::print_status,
    external_api::{
        contracts::{events::Deposited, utils::get_tx_receipt},
        intmax::{
            gnark::{fetch_gnark_proof, gnark_start_prove},
            withdrawal::submit_withdrawal,
        },
    },
    services::utils::await_until_low_gas_price,
    state::{key::Key, state::State},
    utils::config::Settings,
};

pub mod temp;
pub mod witness_generation;

pub async fn withdrawal_task(state: &State, key: &Key, event: Deposited) -> anyhow::Result<()> {
    from_step1(state, key, event).await?;
    Ok(())
}

pub async fn resume_withdrawal_task(state: &State, key: &Key) -> anyhow::Result<()> {
    let status = match temp::WithdrawalStatus::new() {
        Ok(status) => status,
        Err(_) => return Ok(()),
    };
    print_status("Withdrawal: resuming withdrawal");
    match status.next_step {
        temp::WithdrawalStep::Plonky2Prove => from_step2(state, key).await?,
        temp::WithdrawalStep::GnarkStart => from_step3(state, key).await?,
        temp::WithdrawalStep::GnarkGetProof => from_step4(state, key).await?,
        temp::WithdrawalStep::ContractCall => from_step5(state, key).await?,
    }
    Ok(())
}

// Generate witness
async fn from_step1(state: &State, key: &Key, event: Deposited) -> anyhow::Result<()> {
    print_status("Withdrawal: generating withdrawal witness");
    let witness = witness_generation::generate_withdrawa_witness(state, key, event)?;
    let status = temp::WithdrawalStatus {
        next_step: temp::WithdrawalStep::Plonky2Prove,
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
    print_status("Withdrawal: proving with plonky2");
    let mut status = temp::WithdrawalStatus::new()?;
    ensure!(status.next_step == temp::WithdrawalStep::Plonky2Prove);
    ensure!(state.prover.is_some(), "Prover is not initialized");
    let plonky2_proof = state
        .prover
        .as_ref()
        .unwrap()
        .withdrawal_wrapper_processor
        .prove(&status.witness)?;
    status.plonlky2_proof = Some(plonky2_proof.clone());
    status.next_step = temp::WithdrawalStep::GnarkStart;
    status.save()?;
    from_step3(state, key).await?;
    Ok(())
}

// Start Gnark
async fn from_step3(state: &State, key: &Key) -> anyhow::Result<()> {
    print_status("Withdrawal: starting gnark");
    let mut status = temp::WithdrawalStatus::new()?;
    ensure!(status.next_step == temp::WithdrawalStep::GnarkStart);
    let settings = Settings::load()?;
    let withdrawal_address = key.withdrawal_address;
    let prover_url = settings.api.withdrawal_gnark_prover_url.clone();
    let plonky2_proof = status.plonlky2_proof.clone().unwrap();
    let output = gnark_start_prove(&prover_url, withdrawal_address, plonky2_proof).await?;
    status.job_id = Some(output.job_id.clone());
    let now: u64 = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    status.start_query_time = Some(output.estimated_time.unwrap_or(0) / 1000 + now);
    status.next_step = temp::WithdrawalStep::GnarkGetProof;
    status.save()?;
    from_step4(state, key).await?;
    Ok(())
}

// Get Gnark proof
async fn from_step4(state: &State, key: &Key) -> anyhow::Result<()> {
    print_status("Withdrawal: getting gnark proof");
    let mut status = temp::WithdrawalStatus::new()?;
    ensure!(status.next_step == temp::WithdrawalStep::GnarkGetProof);
    let settings = Settings::load()?;
    let prover_url = settings.api.withdrawal_gnark_prover_url.clone();
    let output = fetch_gnark_proof(
        &prover_url,
        status.job_id.as_ref().unwrap(),
        status.start_query_time.unwrap(),
    )
    .await?;
    status.gnark_proof = Some(output.proof.clone());
    status.next_step = temp::WithdrawalStep::ContractCall;
    status.save()?;
    from_step5(state, key).await?;
    Ok(())
}

// Call contract
async fn from_step5(_state: &State, _key: &Key) -> anyhow::Result<()> {
    print_status("Withdrawal: calling contract");
    let status = temp::WithdrawalStatus::new()?;
    ensure!(status.next_step == temp::WithdrawalStep::ContractCall);
    let pis = SimpleWithdrawalPublicInputs {
        deposit_root: status.witness.deposit_root,
        nullifier: status.witness.nullifier,
        recipient: status.witness.recipient,
        token_index: status.witness.deposit_leaf.token_index,
        amount: status.witness.deposit_leaf.amount,
    };
    await_until_low_gas_price().await?;
    let tx_hash = submit_withdrawal(pis, status.gnark_proof.as_ref().unwrap()).await?;
    // delete here because get_tx_receipt may fail, and we don't want to retry this step
    temp::WithdrawalStatus::delete()?;

    print_status(format!("Withdral tx hash: {:?}", tx_hash));
    let tx_reciept = get_tx_receipt(tx_hash).await?;
    ensure!(
        tx_reciept.status == Some(ethers::types::U64::from(1)),
        "Withdrawal transaction failed"
    );
    print_status(format!("Withdrawan with tx hash: {:?}", tx_hash));
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        state::prover::Prover,
        test::{get_dummy_keys, get_dummy_state},
    };

    #[tokio::test]
    #[ignore]
    async fn test_withdrawal() {
        let mut state = get_dummy_state().await;

        let dummy_key = get_dummy_keys();
        let assets_status = state.sync_and_fetch_assets(&dummy_key).await.unwrap();
        let events = assets_status.get_not_withdrawn_events();
        assert!(events.len() > 0);

        let prover = Prover::new();
        state.prover = Some(prover);

        super::withdrawal_task(&state, &dummy_key, events[0].clone())
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_resume_withdrawal() {
        let mut state = get_dummy_state().await;
        state.sync_trees().await.unwrap();
        let dummy_key = get_dummy_keys();
        let prover = Prover::new();
        state.prover = Some(prover);
        super::resume_withdrawal_task(&state, &dummy_key)
            .await
            .unwrap();
    }
}
