use anyhow::{Context, Result};
use std::fs;
use toml::Value;

use crate::types::DependencyType;

pub fn parse_cargo_toml(
    path: &str,
    include_build: bool,
    source_name: &str,
) -> Result<Vec<(String, String, DependencyType, String)>> {
    // Ensure the path is a valid Cargo.toml file
    let path = crate::utils::ensure_cargo_toml_path(path);
    let content = fs::read_to_string(path.as_ref())
        .with_context(|| format!("Failed to read file: {path}"))?;

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
