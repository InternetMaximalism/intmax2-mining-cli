use std::time::UNIX_EPOCH;

use anyhow::ensure;
use mining_circuit::withdrawal::simple_withraw_circuit::SimpleWithdrawalPublicInputs;

use crate::{
    cli::console::print_status,
    config::Settings,
    external_api::{
        contracts::{events::Deposited, utils::get_tx_receipt},
        intmax::{
            gnark::{fetch_gnark_proof, gnark_start_prove},
            withdrawal::submit_withdrawal,
        },
    },
    state::state::State,
};

pub mod temp;
pub mod witness_generation;

pub async fn withdrawal_task(state: &State, event: Deposited) -> anyhow::Result<()> {
    from_step1(state, event).await?;
    Ok(())
}

pub async fn resume_withdrawal_task(state: &State) -> anyhow::Result<()> {
    let status = match temp::WithdrawalStatus::new() {
        Ok(status) => status,
        Err(_) => return Ok(()),
    };
    print_status("Withdrawal: resuming withdrawal");
    match status.next_step {
        temp::WithdrawalStep::Plonky2Prove => from_step2(state).await?,
        temp::WithdrawalStep::GnarkStart => from_step3(state).await?,
        temp::WithdrawalStep::GnarkGetProof => from_step4(state).await?,
        temp::WithdrawalStep::ContractCall => from_step5(state).await?,
    }
    Ok(())
}

// Generate witness
async fn from_step1(state: &State, event: Deposited) -> anyhow::Result<()> {
    print_status("Withdrawal: generating withdrawal witness");
    let witness = witness_generation::generate_withdrawa_witness(state, event)?;
    let status = temp::WithdrawalStatus {
        next_step: temp::WithdrawalStep::Plonky2Prove,
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
    from_step3(state).await?;
    Ok(())
}

// Start Gnark
async fn from_step3(state: &State) -> anyhow::Result<()> {
    print_status("Withdrawal: starting gnark");
    let mut status = temp::WithdrawalStatus::new()?;
    ensure!(status.next_step == temp::WithdrawalStep::GnarkStart);
    let settings = Settings::new()?;
    let withdrawal_address = state.private_data.withdrawal_address.clone();
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
    from_step4(state).await?;
    Ok(())
}

// Get Gnark proof
async fn from_step4(_state: &State) -> anyhow::Result<()> {
    print_status("Withdrawal: getting gnark proof");
    let mut status = temp::WithdrawalStatus::new()?;
    ensure!(status.next_step == temp::WithdrawalStep::GnarkGetProof);
    let settings = Settings::new()?;
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
    from_step5(_state).await?;
    Ok(())
}

// Call contract
async fn from_step5(_state: &State) -> anyhow::Result<()> {
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
    let tx_hash = submit_withdrawal(pis, status.gnark_proof.as_ref().unwrap()).await?;
    print_status(&format!("Withdral tx hash: {:?}", tx_hash));
    let tx_reciept = get_tx_receipt(tx_hash).await?;
    ensure!(
        tx_reciept.status == Some(ethers::types::U64::from(1)),
        "Withdrawal transaction failed"
    );
    temp::WithdrawalStatus::delete()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use intmax2_zkp::{
        common::deposit::get_pubkey_salt_hash, ethereum_types::u256::U256,
        utils::leafable::Leafable as _,
    };

    use crate::{
        external_api::contracts::{
            events::{get_deposited_event, DepositQuery, Deposited},
            int1::get_withdrawal_nullifier_exists,
        },
        state::{prover::Prover, state::State},
        test::get_dummy_state,
        utils::salt::{get_pubkey_from_private_key, get_salt_from_private_key_nonce},
    };

    #[tokio::test]
    async fn test_withdrawal() {
        let mut state = get_dummy_state();
        state.sync_trees().await.unwrap();

        let events = get_not_withdrawn_deposit_events(&state).await.unwrap();
        assert!(events.len() > 0);

        let prover = Prover::new();
        state.prover = Some(prover);

        super::withdrawal_task(&state, events[0].clone())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_resume_withdrawal() {
        let mut state = get_dummy_state();
        state.sync_trees().await.unwrap();
        let prover = Prover::new();
        state.prover = Some(prover);
        super::resume_withdrawal_task(&state).await.unwrap();
    }

    async fn get_not_withdrawn_deposit_events(state: &State) -> anyhow::Result<Vec<Deposited>> {
        let deposit_address = state.private_data.to_addresses().await?.deposit_address;
        let all_senders_deposit_events =
            get_deposited_event(DepositQuery::BySender(deposit_address)).await?;

        // contained in the deposit tree
        let mut contained_deposit_events = Vec::new();

        // not contained in the deposit tree
        let mut not_contained_deposit_events = Vec::new();

        for event in all_senders_deposit_events {
            if let Some(deposit_index) = state.deposit_hash_tree.get_index(event.deposit().hash()) {
                let mut event = event.clone();
                event.deposit_index = Some(deposit_index);
                contained_deposit_events.push(event);
            } else {
                not_contained_deposit_events.push(event);
            }
        }

        // check if there are any deposits that are not withdrawn.
        let deposit_key = state.private_data.deposit_key;
        let mut not_withdrawn_deposit_events = Vec::new();
        for event in contained_deposit_events {
            let salt = get_salt_from_private_key_nonce(deposit_key, event.tx_nonce.unwrap());
            let nullifier = get_pubkey_salt_hash(U256::default(), salt);
            {
                let pubkey = get_pubkey_from_private_key(deposit_key);
                let pubkey_salt_hash = get_pubkey_salt_hash(pubkey, salt);
                assert_eq!(pubkey_salt_hash, event.recipient_salt_hash);
            }
            let is_exists = get_withdrawal_nullifier_exists(nullifier).await?;
            if !is_exists {
                not_withdrawn_deposit_events.push(event);
            }
        }
        Ok(not_withdrawn_deposit_events)
    }
}
