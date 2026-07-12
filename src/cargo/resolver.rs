use anyhow::{Context, Result, bail};
use cargo_metadata::{DependencyKind, MetadataCommand, Package, Source};

use crate::types::{DependencyType, LookupSource, ResolvedDependency};

pub fn resolve_dependencies(
    manifest_path: &str,
    include_build: bool,
    include_workspace: bool,
) -> Result<Vec<ResolvedDependency>> {
    let manifest_path = crate::utils::ensure_cargo_toml_path(manifest_path);
    let metadata = MetadataCommand::new()
        .manifest_path(manifest_path.as_ref())
        .no_deps()
        .exec()
        .with_context(|| format!("Failed to obtain Cargo metadata for {manifest_path}"))?;

    let packages: Vec<&Package> = if include_workspace {
        metadata
            .packages
            .iter()
            .filter(|package| metadata.workspace_members.contains(&package.id))
            .collect()
    } else {
        vec![metadata.root_package().ok_or_else(|| {
            anyhow::anyhow!(
                "The manifest is a virtual workspace; use --workspace to inspect members"
            )
        })?]
    };

    if packages.is_empty() {
        bail!("Cargo metadata did not return any workspace packages");
    }

    let mut dependencies = Vec::new();
    for package in packages {
        for dependency in &package.dependencies {
            let Some(dep_type) = dependency_type(dependency.kind, include_build) else {
                continue;
            };

            if should_skip_path_dependency(dependency.path.is_some(), &dependency.req) {
                continue;
            }

            let display_name = dependency.rename.as_deref().unwrap_or(&dependency.name);
            let package_name = package.name.to_string();
            let source = dependency.target.as_ref().map_or_else(
                || package_name.clone(),
                |target| format!("{package_name} [{target}]"),
            );
            dependencies.push(ResolvedDependency {
                display_name: display_name.to_owned(),
                package_name: dependency.name.clone(),
                version_requirement: dependency.req.to_string(),
                dep_type,
                source,
                lookup_source: lookup_source(
                    dependency.registry.as_deref(),
                    dependency.source.as_ref(),
                ),
            });
        }
    }

    Ok(dependencies)
}

fn should_skip_path_dependency(path_dependency: bool, requirement: &semver::VersionReq) -> bool {
    path_dependency && *requirement == semver::VersionReq::STAR
}

fn lookup_source(registry: Option<&str>, source: Option<&Source>) -> LookupSource {
    if registry.is_some() {
        return LookupSource::Unsupported;
    }

    if source.is_none_or(|source| {
        source.is_crates_io() || source.repr == "sparse+https://index.crates.io/"
    }) {
        LookupSource::CratesIo
    } else {
        LookupSource::Unsupported
    }
}

fn dependency_type(kind: DependencyKind, include_build: bool) -> Option<DependencyType> {
    match kind {
        DependencyKind::Normal => Some(DependencyType::Normal),
        DependencyKind::Development => Some(DependencyType::Dev),
        DependencyKind::Build if include_build => Some(DependencyType::Build),
        DependencyKind::Build | DependencyKind::Unknown => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{lookup_source, resolve_dependencies, should_skip_path_dependency};
    use crate::types::{DependencyType, LookupSource};
    use cargo_metadata::Source;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn resolves_glob_members_aliases_and_target_dependencies() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("cargo-stale-resolver-{unique}"));
        let member = root.join("crates/app");
        fs::create_dir_all(&member).expect("fixture directory must be created");
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\"]\nresolver = \"2\"\n",
        )
        .expect("workspace manifest must be written");
        fs::write(
            member.join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nserde_alias = { package = \"serde\", version = \"1\" }\n\n[target.'cfg(unix)'.dependencies]\nlibc = \"0.2\"\n",
        )
        .expect("member manifest must be written");
        fs::create_dir_all(member.join("src")).expect("fixture source directory must be created");
        fs::write(member.join("src/lib.rs"), "").expect("fixture library must be written");

        let dependencies =
            resolve_dependencies(root.join("Cargo.toml").to_str().unwrap(), false, true)
                .expect("fixture workspace must resolve");
        fs::remove_dir_all(&root).expect("fixture directory must be removed");

        assert!(dependencies.iter().any(|dependency| {
            dependency.display_name == "serde_alias"
                && dependency.package_name == "serde"
                && dependency.dep_type == DependencyType::Normal
        }));
        assert!(dependencies.iter().any(|dependency| {
            dependency.package_name == "libc" && dependency.source.contains("cfg(unix)")
        }));
    }

    #[test]
    fn excludes_non_crates_io_sources_from_network_lookup() {
        assert_eq!(
            lookup_source(
                None,
                Some(&Source {
                    repr: "git+https://example.test/repository".to_owned(),
                })
            ),
            LookupSource::Unsupported
        );
        assert_eq!(
            lookup_source(Some("https://registry.example.test/index"), None),
            LookupSource::Unsupported
        );
    }

    #[test]
    fn keeps_sparse_crates_io_sources_eligible_for_network_lookup() {
        assert_eq!(
            lookup_source(
                None,
                Some(&Source {
                    repr: "sparse+https://index.crates.io/".to_owned(),
                })
            ),
            LookupSource::CratesIo
        );
    }

    #[test]
    fn keeps_versioned_path_dependencies_eligible_for_version_lookup() {
        assert!(should_skip_path_dependency(true, &semver::VersionReq::STAR));
        assert!(!should_skip_path_dependency(
            true,
            &semver::VersionReq::parse("^1").expect("valid requirement")
        ));
    }
}
