use clap::Parser;

#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
pub enum CargoCli {
    Stale(Cli),
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFilter {
    All,
    OutdatedOnly,
}

impl OutputFilter {
    pub fn is_outdated_only(self) -> bool {
        matches!(self, OutputFilter::OutdatedOnly)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DependencyScope {
    Normal,
    IncludeBuildDeps,
}

impl DependencyScope {
    pub fn includes_build_deps(self) -> bool {
        matches!(self, DependencyScope::IncludeBuildDeps)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WorkspaceMode {
    RootOnly,
    IncludeMembers,
}

impl WorkspaceMode {
    pub fn includes_members(self) -> bool {
        matches!(self, WorkspaceMode::IncludeMembers)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OutputVerbosity {
    Normal,
    Verbose,
}

impl OutputVerbosity {
    pub fn is_verbose(self) -> bool {
        matches!(self, OutputVerbosity::Verbose)
    }
}

#[derive(Parser, Debug)]
#[command(version, about = "Check for outdated dependencies in Cargo.toml")]
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    /// Path to Cargo.toml file
    #[arg(short, long, default_value = "Cargo.toml")]
    pub manifest: String,

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

impl Cli {
    pub fn output_filter(&self) -> OutputFilter {
        if self.outdated_only {
            OutputFilter::OutdatedOnly
        } else {
            OutputFilter::All
        }
    }

    pub fn dependency_scope(&self) -> DependencyScope {
        if self.build_deps {
            DependencyScope::IncludeBuildDeps
        } else {
            DependencyScope::Normal
        }
    }

    pub fn workspace_mode(&self) -> WorkspaceMode {
        if self.workspace {
            WorkspaceMode::IncludeMembers
        } else {
            WorkspaceMode::RootOnly
        }
    }

    pub fn output_verbosity(&self) -> OutputVerbosity {
        if self.verbose {
            OutputVerbosity::Verbose
        } else {
            OutputVerbosity::Normal
        }
    }
}
