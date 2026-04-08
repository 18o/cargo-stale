use anyhow::Result;
use crates_index::{HashKind, SparseIndex};
use log::{debug, warn};
use std::collections::HashMap;

const CRATES_IO_URL: &str = "sparse+https://index.crates.io/";

/// Fetch latest versions from the local crates.io sparse index (~/.cargo/registry/index/).
/// Returns a map of crate name → latest version string.
/// Falls back to None for crates not found in the index.
pub fn fetch_versions_from_local_index(
    crate_names: &[String],
) -> Result<HashMap<String, Option<String>>> {
    // Try Stable hash first (Cargo 1.85+), then Legacy hash (older Cargo)
    let index = open_sparse_index()?;

    debug!("Using local crates.io sparse index for version lookup");

    let mut results = HashMap::new();
    for name in crate_names {
        if let Ok(krate) = index.crate_from_cache(name) {
            if let Some(version) = krate.highest_normal_version() {
                let version_str = version.version();
                debug!("Local index: {name} -> {version_str}");
                results.insert(name.clone(), Some(version_str.to_string()));
            } else {
                debug!("Local index: {name} found but no normal version");
                results.insert(name.clone(), None);
            }
        } else {
            warn!("Crate '{name}' not found in local index");
            results.insert(name.clone(), None);
        }
    }

    Ok(results)
}

fn open_sparse_index() -> Result<SparseIndex> {
    let cargo_home = std::env::var("CARGO_HOME").map_or_else(
        |_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            std::path::PathBuf::from(home).join(".cargo")
        },
        std::path::PathBuf::from,
    );

    // Try Stable hash first (Cargo 1.85+), then Legacy hash
    for hash_kind in [HashKind::Stable, HashKind::Legacy] {
        if let Ok(index) =
            SparseIndex::with_path_and_hash_kind(&cargo_home, CRATES_IO_URL, &hash_kind)
        {
            // Verify the index actually works by trying to read a known crate
            if index.crate_from_cache("serde").is_ok() {
                debug!("Opened sparse index with {hash_kind:?} hash");
                return Ok(index);
            }
        }
    }

    // Fallback: let crates-index figure it out
    SparseIndex::new_cargo_default().map_err(Into::into)
}
