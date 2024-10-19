use std::{future::Future, time::Duration};

use log::warn;

const MAX_RETRIES: u32 = 5;
const INITIAL_DELAY: u64 = 1000;

pub async fn with_retry<'a, T, E, F, Fut>(f: F) -> Result<T, E>
where
    E: std::error::Error,
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>> + 'a,
{
    let mut retries = 0;
    let mut delay = Duration::from_millis(INITIAL_DELAY);

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if retries >= MAX_RETRIES {
                    return Err(e);
                }
                warn!(
                    "Attempt {} failed: {}. Retrying in {:?}...",
                    retries + 1,
                    e.to_string(),
                    delay
                );
                std::thread::sleep(delay);
                retries += 1;
                delay *= 2; // Exponential backoff
            }
        }
    }
}
