use anyhow::{Context, Result};
use std::{fs, path::Path};
use toml::Value;

pub fn get_workspace_members(manifest_path: &str) -> Result<Vec<String>> {
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

pub fn get_crate_name(manifest_path: &str) -> String {
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
