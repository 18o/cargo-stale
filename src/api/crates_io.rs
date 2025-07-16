use anyhow::Result;
use log::debug;
use reqwest::Client;

use crate::types::CrateInfo;

pub fn create_client() -> Result<Client> {
    Ok(Client::builder().user_agent("cargo-stale/0.1.0").build()?)
}

pub async fn get_latest_version(client: &Client, crate_name: &str) -> Option<String> {
    debug!("Fetching latest version for crate: {crate_name}");
    let crate_name = crate_name.split_whitespace().next().unwrap_or(crate_name);
    let url = format!("https://crates.io/api/v1/crates/{crate_name}");

    let res = match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<CrateInfo>().await {
                    Ok(info) => Some(info.crate_info.max_version),
                    Err(_) => None,
                }
            } else {
                None
            }
        }
        Err(_) => None,
    };
    debug!("Finished fetching latest version for crate: {crate_name}");
    res
}
