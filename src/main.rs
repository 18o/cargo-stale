#![warn(
    clippy::pedantic,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::correctness
)]

use anyhow::Result;
use clap::Parser;
use std::{
    collections::{HashMap, HashSet},
    env,
};
use tokio::task::JoinSet;

mod api;
mod cargo;
mod cli;
mod output;
mod types;
mod utils;
mod version;

use cli::Cli;
use types::{Dependency, ResolvedDependency, VersionLookup};

const MAX_CONCURRENT_REQUESTS: usize = 20;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting cargo-stale...");
    let cli = parse_cli();
    init_logging(&cli);

    let dependencies = cargo::resolver::resolve_dependencies(
        &cli.manifest,
        cli.dependency_scope().includes_build_deps(),
        cli.workspace_mode().includes_members(),
    )?;
    let client = api::crates_io::create_client()?;

    if cli.output_verbosity().is_verbose() {
        println!("📦 Found {} dependencies to check", dependencies.len());
    }

    let version_cache = fetch_versions(&client, &dependencies, &cli).await;
    let results = build_results(dependencies, &version_cache);
    let has_indeterminate = results
        .iter()
        .any(|dependency| dependency.status().is_indeterminate());

    output::formatter::print_results(&results, &cli);
    if has_indeterminate {
        anyhow::bail!("Some dependencies could not be evaluated");
    }

    Ok(())
}

fn parse_cli() -> Cli {
    parse_cli_from(env::args())
}

fn parse_cli_from(args: impl IntoIterator<Item = String>) -> Cli {
    let args: Vec<String> = args.into_iter().collect();
    if args.get(1).is_some_and(|argument| argument == "stale") {
        Cli::parse_from(std::iter::once(args[0].clone()).chain(args.into_iter().skip(2)))
    } else {
        Cli::parse_from(args)
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
}

async fn fetch_versions(
    client: &reqwest::Client,
    dependencies: &[ResolvedDependency],
    cli: &Cli,
) -> HashMap<String, VersionLookup> {
    let package_names: Vec<String> = dependencies
        .iter()
        .filter(|dependency| dependency.lookup_source == crate::types::LookupSource::CratesIo)
        .map(|dependency| dependency.package_name.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let mut versions = if cli.use_online() {
        HashMap::new()
    } else {
        api::local_index::fetch_versions_from_local_index(&package_names).unwrap_or_else(|error| {
            log::debug!("Local index unavailable: {error}");
            HashMap::new()
        })
    };
    let missing = missing_package_names(&package_names, &versions);

    if cli.output_verbosity().is_verbose() {
        println!(
            "🌐 Fetching {} missing crates from crates.io",
            missing.len()
        );
    }
    versions
        .extend(fetch_missing_versions(client, missing, cli.output_verbosity().is_verbose()).await);
    versions
}

fn missing_package_names(
    package_names: &[String],
    cached_versions: &HashMap<String, VersionLookup>,
) -> Vec<String> {
    package_names
        .iter()
        .filter(|name| !matches!(cached_versions.get(*name), Some(VersionLookup::Found(_))))
        .cloned()
        .collect()
}

async fn fetch_missing_versions(
    client: &reqwest::Client,
    names: Vec<String>,
    verbose: bool,
) -> HashMap<String, VersionLookup> {
    let mut names = names.into_iter();
    let mut tasks = JoinSet::new();
    let mut results = HashMap::new();

    for name in names.by_ref().take(MAX_CONCURRENT_REQUESTS) {
        spawn_version_request(&mut tasks, client.clone(), name, verbose);
    }

    while let Some(task) = tasks.join_next().await {
        match task {
            Ok((name, lookup)) => {
                results.insert(name, lookup);
            }
            Err(error) => log::error!("Version lookup task failed: {error}"),
        }
        if let Some(name) = names.next() {
            spawn_version_request(&mut tasks, client.clone(), name, verbose);
        }
    }

    results
}

fn spawn_version_request(
    tasks: &mut JoinSet<(String, VersionLookup)>,
    client: reqwest::Client,
    name: String,
    verbose: bool,
) {
    tasks.spawn(async move {
        if verbose {
            println!("Fetching latest version for: {name}");
        }
        let lookup = api::crates_io::get_latest_version(&client, &name).await;
        (name, lookup)
    });
}

fn build_results(
    dependencies: Vec<ResolvedDependency>,
    version_cache: &HashMap<String, VersionLookup>,
) -> Vec<Dependency> {
    dependencies
        .into_iter()
        .map(|dependency| Dependency {
            name: dependency.display_name,
            current_version: dependency.version_requirement,
            lookup: version_cache
                .get(&dependency.package_name)
                .cloned()
                .unwrap_or(match dependency.lookup_source {
                    crate::types::LookupSource::CratesIo => VersionLookup::Unavailable,
                    crate::types::LookupSource::Unsupported => VersionLookup::UnsupportedSource,
                }),
            dep_type: dependency.dep_type,
            source: dependency.source,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{missing_package_names, parse_cli_from};
    use crate::types::VersionLookup;
    use std::collections::HashMap;

    #[test]
    fn refetches_only_packages_absent_from_the_local_index() {
        let names = vec!["cached".to_owned(), "missing".to_owned()];
        let cached = HashMap::from([
            (
                "cached".to_owned(),
                VersionLookup::Found("1.0.0".to_owned()),
            ),
            ("missing".to_owned(), VersionLookup::NotFound),
        ]);

        assert_eq!(missing_package_names(&names, &cached), vec!["missing"]);
    }

    #[test]
    fn accepts_the_cargo_subcommand_argument() {
        let cli = parse_cli_from([
            "cargo-stale".to_owned(),
            "stale".to_owned(),
            "--online".to_owned(),
        ]);

        assert!(cli.use_online());
    }
}
