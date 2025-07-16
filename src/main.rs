use anyhow::{Context, Result};
use log::debug;
use reqwest::Client;
use serde::Deserialize;
use std::{env, fs, path::Path};
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

    /// Include workspace members
    #[arg(short, long, default_value = "true")]
    workspace: bool,

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
    source: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum DependencyType {
    Normal,
    Dev,
    Build,
    Workspace,
}

impl std::fmt::Display for DependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyType::Normal => write!(f, ""),
            DependencyType::Dev => write!(f, " (dev)"),
            DependencyType::Build => write!(f, " (build)"),
            DependencyType::Workspace => write!(f, " (workspace)"),
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
                    if current_version.0 == 0 {
                        current_version.1 != latest_parsed.1 || current_version.0 != latest_parsed.0
                    } else {
                        current_version.0 != latest_parsed.0
                    }
                }
                "~" => current_version.0 != latest_parsed.0 || current_version.1 != latest_parsed.1,
                "=" => parsed_req.version != latest_version,
                ">=" | ">" | "<=" | "<" => false,
                _ => parsed_req.version != latest_version,
            }
        } else {
            parsed_req.version != latest_version
        }
    } else {
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
    let args: Vec<String> = env::args().collect();

    let cli = if args.len() > 1 && args[1] == "stale" {
        let mut modified_args = vec![args[0].clone()];
        modified_args.extend_from_slice(&args[2..]);
        Cli::parse_from(modified_args)
    } else {
        Cli::parse()
    };

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

    let mut all_dependencies = Vec::new();

    let main_deps = parse_cargo_toml(&cli.manifest, cli.build_deps, "root")?;
    all_dependencies.extend(main_deps);

    if cli.workspace {
        let workspace_members = get_workspace_members(&cli.manifest)?;
        for member_path in workspace_members {
            if cli.verbose {
                println!("📦 Checking workspace member: {member_path}");
            }

            let member_deps =
                parse_cargo_toml(&member_path, cli.build_deps, &get_crate_name(&member_path))?;
            all_dependencies.extend(member_deps);
        }
    }

    let client = Client::builder().user_agent("cargo-stale/0.1.0").build()?;

    if cli.verbose {
        println!("📦 Found {} dependencies to check", all_dependencies.len());
    }

    let tasks: Vec<_> = all_dependencies
        .into_iter()
        .map(|(name, version, dep_type, source)| {
            let client = client.clone();
            let verbose = cli.verbose;

            tokio::spawn(async move {
                if verbose {
                    println!("Checking {name}{dep_type} from {source} ...");
                }

                let latest_version = get_latest_version(&client, &name).await;

                Dependency {
                    name,
                    current_version: version,
                    latest_version,
                    dep_type,
                    source,
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

fn get_workspace_members(manifest_path: &str) -> Result<Vec<String>> {
    let content = fs::read_to_string(manifest_path)
        .with_context(|| format!("Failed to read file: {manifest_path}"))?;

    let toml: Value = toml::from_str(&content).with_context(|| "Failed to parse Cargo.toml")?;

    let mut members = Vec::new();

    if let Some(workspace) = toml.get("workspace") {
        if let Some(member_list) = workspace.get("members").and_then(|v| v.as_array()) {
            let base_dir = Path::new(manifest_path).parent().unwrap_or(Path::new("."));

            for member in member_list {
                if let Some(member_str) = member.as_str() {
                    let member_path = base_dir.join(member_str).join("Cargo.toml");
                    if member_path.exists() {
                        members.push(member_path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    Ok(members)
}

fn get_crate_name(manifest_path: &str) -> String {
    if let Ok(content) = fs::read_to_string(manifest_path) {
        if let Ok(toml) = toml::from_str::<Value>(&content) {
            if let Some(package) = toml.get("package") {
                if let Some(name) = package.get("name").and_then(|v| v.as_str()) {
                    return name.to_string();
                }
            }
        }
    }

    Path::new(manifest_path)
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

fn parse_cargo_toml(
    path: &str,
    include_build: bool,
    source_name: &str,
) -> Result<Vec<(String, String, DependencyType, String)>> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read file: {path}"))?;

    let toml: Value = toml::from_str(&content).with_context(|| "Failed to parse Cargo.toml")?;

    let mut dependencies = Vec::new();

    let mut workspace_versions = std::collections::HashMap::new();
    if let Some(workspace_deps) = toml
        .get("workspace")
        .and_then(|w| w.get("dependencies"))
        .and_then(|v| v.as_table())
    {
        for (name, value) in workspace_deps {
            if let Some(version) = extract_version_only(value) {
                workspace_versions.insert(name.clone(), version.clone());
                dependencies.push((
                    name.clone(),
                    version,
                    DependencyType::Workspace,
                    source_name.to_string(),
                ));
            }
        }
    }

    // Parse [dependencies]
    if let Some(deps) = toml.get("dependencies").and_then(|v| v.as_table()) {
        for (name, value) in deps {
            if let Some(version) = extract_version_with_workspace(value, &workspace_versions) {
                dependencies.push((
                    name.clone(),
                    version,
                    DependencyType::Normal,
                    source_name.to_string(),
                ));
            }
        }
    }

    // Parse [dev-dependencies]
    if let Some(dev_deps) = toml.get("dev-dependencies").and_then(|v| v.as_table()) {
        for (name, value) in dev_deps {
            if let Some(version) = extract_version_with_workspace(value, &workspace_versions) {
                dependencies.push((
                    name.clone(),
                    version,
                    DependencyType::Dev,
                    source_name.to_string(),
                ));
            }
        }
    }

    // Parse [build-dependencies] (optional)
    if include_build {
        if let Some(build_deps) = toml.get("build-dependencies").and_then(|v| v.as_table()) {
            for (name, value) in build_deps {
                if let Some(version) = extract_version_with_workspace(value, &workspace_versions) {
                    dependencies.push((
                        name.clone(),
                        version,
                        DependencyType::Build,
                        source_name.to_string(),
                    ));
                }
            }
        }
    }

    Ok(dependencies)
}

fn extract_version_only(value: &Value) -> Option<String> {
    match value {
        Value::String(version) => Some(version.clone()),
        Value::Table(table) => table
            .get("version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        _ => None,
    }
}

fn extract_version_with_workspace(
    value: &Value,
    _workspace_versions: &std::collections::HashMap<String, String>,
) -> Option<String> {
    match value {
        Value::String(version) => Some(version.clone()),
        Value::Table(table) => {
            if table.get("workspace").and_then(|v| v.as_bool()) == Some(true) {
                None
            } else if let Some(version) = table.get("version").and_then(|v| v.as_str()) {
                Some(version.to_string())
            } else if table.contains_key("path") {
                // ignore path dependencies
                None
            } else if table.contains_key("git") {
                // ignore git dependencies
                None
            } else {
                None
            }
        }
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
    println!("{:-<110}", "");
    println!(
        "{:<35} {:<20} {:<20} {:<15} {:<15}",
        "Dependency", "Current Version", "Latest Version", "Status", "Source"
    );
    println!("{:-<110}", "");

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
            "{:<35} {:<20} {:<20} {:<15} {:<15}",
            name_with_type, dep.current_version, latest_display, status, dep.source
        );
    }

    println!("{:-<110}", "");

    if outdated_count > 0 {
        println!("⚠️  Found {outdated_count} outdated dependencies");
        if cli.verbose {
            println!("💡 Use 'cargo update <crate_name>' to update specific dependencies");
        }
    } else if !cli.outdated_only {
        println!("🎉 All dependencies are up to date!");
    }
}
