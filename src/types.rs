use crate::version::comparison::is_version_outdated;
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
    pub latest_version: Option<String>,
    pub dep_type: DependencyType,
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub fn is_outdated(&self) -> bool {
        if let Some(latest) = &self.latest_version {
            is_version_outdated(&self.current_version, latest)
        } else {
            false
        }
    }
}
