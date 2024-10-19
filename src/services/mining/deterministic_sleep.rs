use std::{
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use ethers::types::Address;
use log::info;
use rand::{Rng as _, SeedableRng as _};
use rand_chacha::ChaCha20Rng;

use crate::{
    cli::console::print_log,
    external_api::contracts::events::{
        get_latest_deposit_timestamp, get_latest_withdrawal_timestamp,
    },
    utils::{config::Settings, encryption::keccak256_hash},
};

/// Random sleep before deposit to improve privacy.
pub async fn sleep_before_deposit(withdrawal_address: Address) -> anyhow::Result<()> {
    let last_withdrawal_time = get_latest_withdrawal_timestamp(withdrawal_address).await?;
    info!("last_withdrawal_time: {:?}", last_withdrawal_time);
    if last_withdrawal_time.is_none() {
        return Ok(()); // no withdrawal yet
    }
    let last_withdrawal_time = last_withdrawal_time.unwrap();
    let sleep_time = determin_sleep_time(last_withdrawal_time, withdrawal_address, "deposit");
    let target_time = last_withdrawal_time + sleep_time;
    sleep_if_needed(target_time, true).await;
    Ok(())
}

/// Random sleep before withdrawal to improve privacy.  
pub async fn sleep_before_withdrawal(deposit_address: Address) -> anyhow::Result<()> {
    let last_deposit_time = get_latest_deposit_timestamp(deposit_address).await?;
    info!("last_deposit_time: {:?}", last_deposit_time);
    if last_deposit_time.is_none() {
        return Ok(()); // no deposit yet
    }
    let last_deposit_time = last_deposit_time.unwrap();
    let sleep_time = determin_sleep_time(last_deposit_time, deposit_address, "withdrawal");
    let target_time = last_deposit_time + sleep_time;
    sleep_if_needed(target_time, false).await;
    Ok(())
}

async fn sleep_if_needed(target_time: u64, is_deposit: bool) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    if now >= target_time {
        info!("No need to sleep");
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
    sleep(Duration::from_secs(sleep_from_now));
}

fn determin_sleep_time(last_time: u64, address: Address, random_nonce: &'static str) -> u64 {
    let seed_str = format!("{}{}{}", last_time, address, random_nonce);
    let seed_hash = keccak256_hash(&seed_str);
    let mut rng = ChaCha20Rng::from_seed(seed_hash);
    let settings = Settings::load().expect("Failed to load settings");
    rng.gen_range(
        settings.service.mining_min_cooldown_in_sec..settings.service.mining_max_cooldown_in_sec,
    )
}
