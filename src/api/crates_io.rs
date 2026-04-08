use anyhow::Result;
use log::{debug, warn};
use reqwest::Client;
use std::time::Duration;

use crate::types::CrateInfo;

const MAX_RETRIES: u32 = 2;
const RETRY_DELAY_MS: u64 = 500;

pub fn create_client() -> Result<Client> {
    Ok(Client::builder()
        .user_agent("cargo-stale/0.1.6")
        .pool_max_idle_per_host(20)
        .pool_idle_timeout(Some(Duration::from_secs(30)))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(15))
        .build()?)
}

pub async fn get_latest_version(client: &Client, crate_name: &str) -> Option<String> {
    let crate_name = crate_name.split_whitespace().next().unwrap_or(crate_name);
    let url = format!("https://crates.io/api/v1/crates/{crate_name}");

    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            debug!("Retry {attempt}/{MAX_RETRIES} for crate '{crate_name}'");
            tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS * u64::from(attempt))).await;
        }

        match client.get(&url).send().await {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    match response.json::<CrateInfo>().await {
                        Ok(info) => return Some(info.crate_info.max_version),
                        Err(e) => {
                            warn!("Failed to parse response for crate '{crate_name}': {e}");
                            return None;
                        }
                    }
                } else if status.as_u16() == 429 {
                    warn!("Rate limited (429) for crate '{crate_name}', retrying...");
                    continue;
                } else if status.is_server_error() {
                    warn!("Server error {status} for crate '{crate_name}', retrying...");
                    continue;
                }
                warn!("HTTP {status} for crate '{crate_name}'");
                return None;
            }
            Err(e) => {
                if e.is_timeout() || e.is_connect() {
                    warn!("Connection error for crate '{crate_name}': {e}, retrying...");
                    continue;
                }
                warn!("Request failed for crate '{crate_name}': {e}");
                return None;
            }
        }
    }

    warn!("All retries exhausted for crate '{crate_name}'");
    None
}
