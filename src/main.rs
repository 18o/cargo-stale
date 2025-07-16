use anyhow::{Context, Result};
use log::debug;
use reqwest::Client;
use serde::Deserialize;
use std::{env, fs};
use toml::Value;

use clap::Parser;

#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum CargoCli {
    Stale(Cli),
}

#[derive(Parser, Debug)]
#[command(version, about = "Check for outdated dependencies in Cargo.toml")]
struct Cli {
    /// Path to Cargo.toml file
    #[arg(short, long, default_value = "Cargo.toml")]
    manifest: String,

    /// Show only outdated dependencies
    #[arg(short, long)]
    outdated_only: bool,

    /// Include build dependencies
    #[arg(short, long)]
    build_deps: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Deserialize)]
struct CrateInfo {
    #[serde(rename = "crate")]
    crate_info: CrateDetails,
}

#[derive(Debug, Deserialize)]
struct CrateDetails {
    max_version: String,
}

#[derive(Debug)]
struct Dependency {
    name: String,
    current_version: String,
    latest_version: Option<String>,
    dep_type: DependencyType,
}

#[derive(Debug)]
enum DependencyType {
    Normal,
    Dev,
    Build,
}

impl std::fmt::Display for DependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyType::Normal => write!(f, ""),
            DependencyType::Dev => write!(f, " (dev)"),
            DependencyType::Build => write!(f, " (build)"),
        }
    }
}

impl Dependency {
    fn is_outdated(&self) -> bool {
        if let Some(latest) = &self.latest_version {
            is_version_outdated(&self.current_version, latest)
        } else {
            false
        }
    }
}

fn is_version_outdated(current_req: &str, latest_version: &str) -> bool {
    let current_req = current_req.trim();

    if let Some(parsed_req) = parse_version_requirement(current_req) {
        if let (Some(current_version), Some(latest_parsed)) = (
            parse_simple_version(&parsed_req.version),
            parse_simple_version(latest_version),
        ) {
            match parsed_req.operator.as_str() {
                "^" | "" => {
                    // Caret requirements: ^0.10 allows >=0.10.0 but <0.11.0
                    // ^1.2 allows >=1.2.0 but <2.0.0
                    if current_version.0 == 0 {
                        // For 0.x versions, caret is more restrictive
                        current_version.1 != latest_parsed.1 || current_version.0 != latest_parsed.0
                    } else {
                        // For 1.x+ versions, only major version matters
                        current_version.0 != latest_parsed.0
                    }
                }
                "~" => {
                    // Tilde requirements: ~0.10 allows >=0.10.0 but <0.11.0
                    // ~1.2 allows >=1.2.0 but <1.3.0
                    current_version.0 != latest_parsed.0 || current_version.1 != latest_parsed.1
                }
                "=" => {
                    // Exact requirements: =0.10.0 allows exactly 0.10.0
                    parsed_req.version != latest_version
                }
                ">=" | ">" | "<=" | "<" => {
                    // For inequality operators, we don't consider them outdated
                    // since they specify ranges rather than specific versions
                    false
                }
                _ => {
                    // Fallback to string comparison
                    parsed_req.version != latest_version
                }
            }
        } else {
            // Fallback to string comparison if parsing fails
            parsed_req.version != latest_version
        }
    } else {
        // Fallback to string comparison
        current_req != latest_version
    }
}

#[derive(Debug)]
struct VersionRequirement {
    operator: String,
    version: String,
}

fn parse_version_requirement(req: &str) -> Option<VersionRequirement> {
    let req = req.trim();

    if let Some(version) = req.strip_prefix("^") {
        Some(VersionRequirement {
            operator: "^".to_string(),
            version: version.to_string(),
        })
    } else if let Some(version) = req.strip_prefix("~") {
        Some(VersionRequirement {
            operator: "~".to_string(),
            version: version.to_string(),
        })
    } else if let Some(version) = req.strip_prefix("=") {
        Some(VersionRequirement {
            operator: "=".to_string(),
            version: version.to_string(),
        })
    } else if let Some(version) = req.strip_prefix(">=") {
        Some(VersionRequirement {
            operator: ">=".to_string(),
            version: version.to_string(),
        })
    } else if let Some(version) = req.strip_prefix("<=") {
        Some(VersionRequirement {
            operator: "<=".to_string(),
            version: version.to_string(),
        })
    } else if let Some(version) = req.strip_prefix(">") {
        Some(VersionRequirement {
            operator: ">".to_string(),
            version: version.to_string(),
        })
    } else if let Some(version) = req.strip_prefix("<") {
        Some(VersionRequirement {
            operator: "<".to_string(),
            version: version.to_string(),
        })
    } else {
        Some(VersionRequirement {
            operator: "^".to_string(),
            version: req.to_string(),
        })
    }
}

fn parse_simple_version(version: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 3 {
        if let (Ok(major), Ok(minor), Ok(patch)) = (
            parts[0].parse::<u32>(),
            parts[1].parse::<u32>(),
            parts[2].parse::<u32>(),
        ) {
            return Some((major, minor, patch));
        }
    } else if parts.len() == 2 {
        if let (Ok(major), Ok(minor)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
            return Some((major, minor, 0));
        }
    } else if parts.len() == 1 {
        if let Ok(major) = parts[0].parse::<u32>() {
            return Some((major, 0, 0));
        }
    }
    None
}

#[tokio::main]
async fn main() -> Result<()> {
    // check if the program is run as a cargo subcommand or directly
    let args: Vec<String> = env::args().collect();

    let cli = if args.len() > 1 && args[1] == "stale" {
        let mut modified_args = vec![args[0].clone()];
        modified_args.extend_from_slice(&args[2..]);

        Cli::parse_from(modified_args)
    } else {
        Cli::parse()
    };

    println!("🚀 Starting cargo-stale...");
    println!("📦 Checking Cargo.toml at: {}", cli.manifest);

    // set log level based on verbose option
    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
        println!("🔍 Checking dependency versions...");
        println!("📁 Cargo.toml path: {}", cli.manifest);
    } else {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Warn)
            .init();
    }

    let dependencies = parse_cargo_toml(&cli.manifest, cli.build_deps)?;
    let client = Client::builder().user_agent("cargo-stale/0.1.0").build()?;

    if cli.verbose {
        println!("📦 Found {} dependencies to check", dependencies.len());
    }

    let tasks: Vec<_> = dependencies
        .into_iter()
        .map(|(name, version, dep_type)| {
            let client = client.clone();
            let verbose = cli.verbose;

            tokio::spawn(async move {
                if verbose {
                    println!("Checking {name}{dep_type} ...");
                }

                let latest_version = get_latest_version(&client, &name).await;

                Dependency {
                    name,
                    current_version: version,
                    latest_version,
                    dep_type,
                }
            })
        })
        .collect();

    let mut results = Vec::new();
    for task in tasks {
        match task.await {
            Ok(dependency) => results.push(dependency),
            Err(e) => eprintln!("Task failed: {e}"),
        }
    }

    if cli.verbose {
        println!("✅ Completed checking all dependencies");
    }

    print_results(&results, &cli);

    Ok(())
}

fn parse_cargo_toml(
    path: &str,
    include_build: bool,
) -> Result<Vec<(String, String, DependencyType)>> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read file: {path}"))?;

    let toml: Value = toml::from_str(&content).with_context(|| "Failed to parse Cargo.toml")?;

    let mut dependencies = Vec::new();

    // Parse [dependencies]
    if let Some(deps) = toml.get("dependencies").and_then(|v| v.as_table()) {
        for (name, value) in deps {
            if let Some(version) = extract_version(value) {
                dependencies.push((name.clone(), version, DependencyType::Normal));
            }
        }
    }

    // Parse [dev-dependencies]
    if let Some(dev_deps) = toml.get("dev-dependencies").and_then(|v| v.as_table()) {
        for (name, value) in dev_deps {
            if let Some(version) = extract_version(value) {
                dependencies.push((name.clone(), version, DependencyType::Dev));
            }
        }
    }

    // Parse [build-dependencies] (optional)
    if include_build {
        if let Some(build_deps) = toml.get("build-dependencies").and_then(|v| v.as_table()) {
            for (name, value) in build_deps {
                if let Some(version) = extract_version(value) {
                    dependencies.push((name.clone(), version, DependencyType::Build));
                }
            }
        }
    }

    Ok(dependencies)
}

fn extract_version(value: &Value) -> Option<String> {
    match value {
        Value::String(version) => Some(version.clone()),
        Value::Table(table) => table
            .get("version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        _ => None,
    }
}

async fn get_latest_version(client: &Client, crate_name: &str) -> Option<String> {
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

fn print_results(results: &[Dependency], cli: &Cli) {
    let filtered_results: Vec<_> = if cli.outdated_only {
        results.iter().filter(|dep| dep.is_outdated()).collect()
    } else {
        results.iter().collect()
    };

    if filtered_results.is_empty() {
        if cli.outdated_only {
            println!("🎉 No outdated dependencies found!");
        } else {
            println!("❌ No dependencies found");
        }
        return;
    }

    println!("\n📊 Dependency Check Results:");
    println!("{:-<90}", "");
    println!(
        "{:<35} {:<20} {:<20} {:<10}",
        "Dependency", "Current Version", "Latest Version", "Status"
    );
    println!("{:-<90}", "");

    let mut outdated_count = 0;

    for dep in &filtered_results {
        let status = match &dep.latest_version {
            Some(_latest) => {
                if dep.is_outdated() {
                    outdated_count += 1;
                    "🔴 Outdated"
                } else {
                    "✅ Latest"
                }
            }
            None => "❓ Unknown",
        };

        let latest_display = dep.latest_version.as_deref().unwrap_or("N/A");
        let name_with_type = format!("{}{}", dep.name, dep.dep_type);

        println!(
            "{:<35} {:<20} {:<20} {:<10}",
            name_with_type, dep.current_version, latest_display, status
        );
    }

    println!("{:-<90}", "");

    if outdated_count > 0 {
        println!("⚠️  Found {outdated_count} outdated dependencies");
        if cli.verbose {
            println!("💡 Use 'cargo update <crate_name>' to update specific dependencies");
        }
    } else if !cli.outdated_only {
        println!("🎉 All dependencies are up to date!");
    }
}
