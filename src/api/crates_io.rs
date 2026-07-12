use anyhow::Result;
use log::warn;
use reqwest::Client;
use std::time::{Duration, SystemTime};

use crate::types::{CrateInfo, VersionLookup};

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

pub async fn get_latest_version(client: &Client, crate_name: &str) -> VersionLookup {
    let url = format!("https://crates.io/api/v1/crates/{crate_name}");

    for attempt in 0..=MAX_RETRIES {
        match client.get(&url).send().await {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    match response.json::<CrateInfo>().await {
                        Ok(info) => return VersionLookup::Found(info.crate_info.max_version),
                        Err(e) => {
                            warn!("Failed to parse response for crate '{crate_name}': {e}");
                            return VersionLookup::Unavailable;
                        }
                    }
                } else if status.as_u16() == 429 {
                    warn!("Rate limited (429) for crate '{crate_name}', retrying...");
                    if attempt < MAX_RETRIES {
                        retry_after(response.headers(), attempt).await;
                        continue;
                    }
                } else if status.is_server_error() {
                    warn!("Server error {status} for crate '{crate_name}', retrying...");
                    if attempt < MAX_RETRIES {
                        retry_after(response.headers(), attempt).await;
                        continue;
                    }
                }
                if status.as_u16() == 404 {
                    return VersionLookup::NotFound;
                }
                warn!("HTTP {status} for crate '{crate_name}'");
                return VersionLookup::Unavailable;
            }
            Err(e) => {
                if e.is_timeout() || e.is_connect() {
                    warn!("Connection error for crate '{crate_name}': {e}, retrying...");
                    if attempt < MAX_RETRIES {
                        tokio::time::sleep(retry_delay(None, attempt)).await;
                        continue;
                    }
                }
                warn!("Request failed for crate '{crate_name}': {e}");
                return VersionLookup::Unavailable;
            }
        }
    }

    warn!("All retries exhausted for crate '{crate_name}'");
    VersionLookup::Unavailable
}

async fn retry_after(headers: &reqwest::header::HeaderMap, attempt: u32) {
    tokio::time::sleep(retry_delay(
        headers.get(reqwest::header::RETRY_AFTER),
        attempt,
    ))
    .await;
}

fn retry_delay(retry_after: Option<&reqwest::header::HeaderValue>, attempt: u32) -> Duration {
    let fallback = Duration::from_millis(RETRY_DELAY_MS * u64::from(attempt + 1));
    retry_after_delay(retry_after, SystemTime::now()).unwrap_or(fallback)
}

fn retry_after_delay(
    retry_after: Option<&reqwest::header::HeaderValue>,
    now: SystemTime,
) -> Option<Duration> {
    retry_after
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            value
                .parse::<u64>()
                .ok()
                .map(Duration::from_secs)
                .or_else(|| {
                    httpdate::parse_http_date(value)
                        .ok()
                        .map(|date| date.duration_since(now).unwrap_or_default())
                })
        })
}

#[cfg(test)]
mod tests {
    use super::{retry_after_delay, retry_delay};
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn honors_numeric_retry_after_headers() {
        let value = reqwest::header::HeaderValue::from_static("3");

        assert_eq!(retry_delay(Some(&value), 0), Duration::from_secs(3));
    }

    #[test]
    fn honors_http_date_retry_after_headers() {
        let value = reqwest::header::HeaderValue::from_static("Thu, 01 Jan 1970 00:00:03 GMT");

        assert_eq!(
            retry_after_delay(Some(&value), UNIX_EPOCH),
            Some(Duration::from_secs(3))
        );
    }
}
