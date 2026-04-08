#![warn(
    clippy::pedantic,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::correctness
)]

use anyhow::Result;
use clap::Parser;
use std::{collections::HashMap, env, sync::Arc};
use tokio::sync::Semaphore;

mod api;
mod cargo;
mod cli;
mod output;
mod types;
mod utils;
mod version;

use cli::Cli;
use types::Dependency;

const MAX_CONCURRENT_REQUESTS: usize = 20;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting cargo-stale...");
    let cli = parse_cli();
    init_logging(&cli);

    let all_dependencies = collect_dependencies(&cli)?;
    let client = api::crates_io::create_client()?;

    if cli.output_verbosity().is_verbose() {
        println!("📦 Found {} dependencies to check", all_dependencies.len());
    }

    let version_cache = fetch_versions(&client, &all_dependencies, &cli).await?;

    let results = build_results(all_dependencies, &version_cache);

    if cli.output_verbosity().is_verbose() {
        println!("✅ Completed processing all dependencies");
    }

    output::formatter::print_results(&results, &cli);

    Ok(())
}

fn parse_cli() -> Cli {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "stale" {
        let mut modified_args = vec![args[0].clone()];
        modified_args.extend_from_slice(&args[2..]);
        Cli::parse_from(modified_args)
    } else {
        Cli::parse()
    }
}

fn init_logging(cli: &Cli) {
    let level = if cli.output_verbosity().is_verbose() {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Warn
    };
    env_logger::Builder::from_default_env()
        .filter_level(level)
        .init();

    if cli.output_verbosity().is_verbose() {
        println!("🔍 Checking dependency versions...");
        println!("📁 Cargo.toml path: {}", cli.manifest);
    }
}

fn collect_dependencies(cli: &Cli) -> Result<Vec<(String, String, types::DependencyType, String)>> {
    let mut all_deps = Vec::new();

    let main_deps = cargo::parser::parse_cargo_toml(
        &cli.manifest,
        cli.dependency_scope().includes_build_deps(),
        "root",
    )?;
    all_deps.extend(main_deps);

    if cli.workspace_mode().includes_members() {
        let workspace_members = cargo::workspace::get_workspace_members(&cli.manifest)?;
        for member_path in workspace_members {
            if cli.output_verbosity().is_verbose() {
                println!("📦 Checking workspace member: {member_path}");
            }
            let member_deps = cargo::parser::parse_cargo_toml(
                &member_path,
                cli.dependency_scope().includes_build_deps(),
                &cargo::workspace::get_crate_name(&member_path),
            )?;
            all_deps.extend(member_deps);
        }
    }

    Ok(all_deps)
}

async fn fetch_versions(
    client: &reqwest::Client,
    all_dependencies: &[(String, String, types::DependencyType, String)],
    cli: &Cli,
) -> Result<HashMap<String, Option<String>>> {
    let unique_names: Vec<String> = {
        let mut seen = std::collections::HashSet::new();
        all_dependencies
            .iter()
            .map(|(name, _, _, _)| name.clone())
            .filter(|name| seen.insert(name.clone()))
            .collect()
    };

    if cli.output_verbosity().is_verbose() {
        println!("📦 Unique crates to check: {}", unique_names.len());
    }

    // Try local index first (unless --online is specified)
    if !cli.use_online() {
        match api::local_index::fetch_versions_from_local_index(&unique_names) {
            Ok(cache) => {
                let found = cache.values().filter(|v| v.is_some()).count();
                if cli.output_verbosity().is_verbose() {
                    println!("📚 Local index: resolved {}/{} crates", found, unique_names.len());
                }
                if found > 0 {
                    return Ok(cache);
                }
            }
            Err(e) => {
                if cli.output_verbosity().is_verbose() {
                    println!("⚠️  Local index unavailable: {e}, falling back to online mode");
                }
            }
        }
    } else if cli.output_verbosity().is_verbose() {
        println!("🌐 Online mode: fetching from crates.io API");
    }

    // Fallback: online HTTP API with concurrency limiter
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));
    let version_tasks: Vec<_> = unique_names
        .into_iter()
        .map(|name| {
            let client = client.clone();
            let sem = semaphore.clone();
            let verbose = cli.output_verbosity().is_verbose();

            tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                if verbose {
                    println!("Fetching latest version for: {name}");
                }
                let latest_version = api::crates_io::get_latest_version(&client, &name).await;
                (name, latest_version)
            })
        })
        .collect();

    let mut version_cache = HashMap::new();
    for task in version_tasks {
        match task.await {
            Ok((name, version)) => {
                version_cache.insert(name, version);
            }
            Err(e) => eprintln!("Version fetch task failed: {e}"),
        }
    }

    if cli.output_verbosity().is_verbose() {
        println!("✅ Completed fetching all versions");
    }

    Ok(version_cache)
}

fn build_results(
    all_dependencies: Vec<(String, String, types::DependencyType, String)>,
    version_cache: &HashMap<String, Option<String>>,
) -> Vec<Dependency> {
    all_dependencies
        .into_iter()
        .map(|(name, current_version, dep_type, source)| Dependency {
            latest_version: version_cache.get(&name).cloned().flatten(),
            name,
            current_version,
            dep_type,
            source,
        })
        .collect()
}
