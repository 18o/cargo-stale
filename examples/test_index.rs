fn main() {
    let url = "sparse+https://index.crates.io/";
    let home = std::path::PathBuf::from("/Users/john/.cargo");
    
    println!("=== Legacy hash ===");
    if let Ok(idx) = crates_index::SparseIndex::new_cargo_default() {
        match idx.crate_from_cache("anyhow") {
            Ok(k) => println!("anyhow -> {:?}", k.highest_normal_version().map(|v| v.version())),
            Err(e) => println!("anyhow error: {e}"),
        }
    } else {
        println!("Failed to create legacy index");
    }
    
    println!("=== Stable hash ===");
    if let Ok(idx) = crates_index::SparseIndex::with_path_and_hash_kind(
        &home, url, &crates_index::HashKind::Stable
    ) {
        match idx.crate_from_cache("anyhow") {
            Ok(k) => println!("anyhow -> {:?}", k.highest_normal_version().map(|v| v.version())),
            Err(e) => println!("anyhow error: {e}"),
        }
    } else {
        println!("Failed to create stable index");
    }
}
