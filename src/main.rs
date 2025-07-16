use anyhow::Result;
use clap::Parser;
use std::{collections::HashMap, env};

mod api;
mod cargo;
mod cli;
mod output;
mod types;
mod version;

use cli::Cli;
use types::Dependency;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting cargo-stale...");
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

    // Parse main Cargo.toml
    let main_deps = cargo::parser::parse_cargo_toml(&cli.manifest, cli.build_deps, "root")?;
    all_dependencies.extend(main_deps);

    // Parse workspace members if enabled
    if cli.workspace {
        let workspace_members = cargo::workspace::get_workspace_members(&cli.manifest)?;
        for member_path in workspace_members {
            if cli.verbose {
                println!("📦 Checking workspace member: {member_path}");
            }

            let member_deps = cargo::parser::parse_cargo_toml(
                &member_path,
                cli.build_deps,
                &cargo::workspace::get_crate_name(&member_path),
            )?;
            all_dependencies.extend(member_deps);
        }
    }

    let client = api::crates_io::create_client()?;

    if cli.verbose {
        println!("📦 Found {} dependencies to check", all_dependencies.len());
    }

    let mut unique_crates = std::collections::HashSet::new();
    for (name, _, _, _) in &all_dependencies {
        unique_crates.insert(name.clone());
    }

    if cli.verbose {
        println!("📦 Unique crates to check: {}", unique_crates.len());
    }

    let version_tasks: Vec<_> = unique_crates
        .into_iter()
        .map(|name| {
            let client = client.clone();
            let verbose = cli.verbose;

            tokio::spawn(async move {
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

    if cli.verbose {
        println!("✅ Completed fetching all versions");
    }

    let mut results = Vec::new();
    for (name, current_version, dep_type, source) in all_dependencies {
        let latest_version = version_cache.get(&name).cloned().flatten();

        results.push(Dependency {
            name,
            current_version,
            latest_version,
            dep_type,
            source,
        });
    }

    if cli.verbose {
        println!("✅ Completed processing all dependencies");
    }

    output::formatter::print_results(&results, &cli);

    Ok(())
}
