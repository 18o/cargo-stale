use crate::version::comparison::VersionStatus;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CrateInfo {
    #[serde(rename = "crate")]
    pub crate_info: CrateDetails,
}

#[derive(Debug, Deserialize)]
pub struct CrateDetails {
    pub max_version: String,
}

#[derive(Debug)]
pub struct Dependency {
    pub name: String,
    pub current_version: String,
    pub lookup: VersionLookup,
    pub dep_type: DependencyType,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionLookup {
    Found(String),
    NotFound,
    Unavailable,
    UnsupportedSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDependency {
    pub display_name: String,
    pub package_name: String,
    pub version_requirement: String,
    pub dep_type: DependencyType,
    pub source: String,
    pub lookup_source: LookupSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookupSource {
    CratesIo,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyType {
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
    pub fn status(&self) -> VersionStatus {
        match &self.lookup {
            VersionLookup::Found(latest) => {
                crate::version::comparison::evaluate_requirement(&self.current_version, latest)
            }
            VersionLookup::NotFound => VersionStatus::NotFound,
            VersionLookup::Unavailable => VersionStatus::Unavailable,
            VersionLookup::UnsupportedSource => VersionStatus::UnsupportedSource,
        }
    }

    pub fn is_outdated(&self) -> bool {
        self.status().is_outdated()
    }
}
