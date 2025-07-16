use clap::Parser;

#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
pub enum CargoCli {
    Stale(Cli),
}

#[derive(Parser, Debug)]
#[command(version, about = "Check for outdated dependencies in Cargo.toml")]
pub struct Cli {
    /// Path to Cargo.toml file
    #[arg(short, long, default_value = "Cargo.toml")]
    pub manifest: String,

    /// Show only outdated dependencies
    #[arg(short, long)]
    pub outdated_only: bool,

    /// Include build dependencies
    #[arg(short, long)]
    pub build_deps: bool,

    /// Include workspace members
    #[arg(short, long, default_value = "true")]
    pub workspace: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}
