use alloy::primitives::Address;
use log::info;
use rand::{Rng as _, SeedableRng as _};
use rand_chacha::ChaCha20Rng;

use crate::{
    cli::console::print_log,
    external_api::graph::client::GraphClient,
    utils::{config::Settings, encryption::keccak256_hash, time::sleep_for},
};

/// Random sleep before deposit to improve privacy.
pub async fn sleep_before_deposit(
    graph_client: &GraphClient,
    withdrawal_address: Address,
) -> anyhow::Result<()> {
    let last_withdrawal_time = graph_client
        .get_latest_withdrawal_timestamp(withdrawal_address)
        .await?;
    info!("last_withdrawal_time: {:?}", last_withdrawal_time);
    if last_withdrawal_time.is_none() {
        return Ok(()); // no withdrawal yet
    }
    let last_withdrawal_time = last_withdrawal_time.unwrap();
    let sleep_time = determine_sleep_time(last_withdrawal_time, withdrawal_address, "deposit");
    let target_time = last_withdrawal_time + sleep_time;
    sleep_if_needed(target_time, true).await;
    Ok(())
}

/// Random sleep before withdrawal to improve privacy.  
pub async fn sleep_before_withdrawal(
    graph_client: &GraphClient,
    deposit_address: Address,
) -> anyhow::Result<()> {
    let last_deposit_time: Option<u64> = graph_client
        .get_latest_deposit_timestamp(deposit_address)
        .await?;
    info!("last_deposit_time: {:?}", last_deposit_time);
    if last_deposit_time.is_none() {
        return Ok(()); // no deposit yet
    }
    let last_deposit_time = last_deposit_time.unwrap();
    let sleep_time = determine_sleep_time(last_deposit_time, deposit_address, "withdrawal");
    let target_time = last_deposit_time + sleep_time;
    sleep_if_needed(target_time, false).await;
    Ok(())
}

async fn sleep_if_needed(target_time: u64, is_deposit: bool) {
    log::info!(
        "sleep_if_needed: target_time: {}, is_deposit: {}",
        target_time,
        is_deposit
    );
    let now = chrono::Utc::now().timestamp() as u64;
    if now >= target_time {
        info!("No need to sleep: now={}, target_time={}", now, target_time);
        return; // no need to sleep
    }
    let sleep_from_now = target_time - now;
    let sleep_until_chrono =
        chrono::Local::now() + chrono::Duration::seconds(sleep_from_now as i64);
    let next_step = if is_deposit { "deposit" } else { "withdrawal" };
    print_log(format!(
        "Next {} will start at {}.",
        next_step,
        sleep_until_chrono.format("%Y-%m-%d %H:%M:%S"),
    ));
    sleep_for(sleep_from_now);
}

fn determine_sleep_time(last_time: u64, address: Address, random_nonce: &'static str) -> u64 {
    let seed_str = format!("{}{}{}", last_time, address, random_nonce);
    let seed_hash = keccak256_hash(&seed_str);
    let mut rng = ChaCha20Rng::from_seed(seed_hash);
    let settings = Settings::load().expect("Failed to load settings");
    rng.gen_range(
        settings.service.mining_min_cooldown_in_sec..settings.service.mining_max_cooldown_in_sec,
    )
}
