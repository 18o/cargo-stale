use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: Option<u32>,
    pub patch: Option<u32>,
    pub pre_release: Option<String>,
    pub build: Option<String>,
}

impl Version {
    pub fn parse(version_str: &str) -> Option<Self> {
        let version_str = version_str.trim();

        let (core_version, pre_and_build) = if let Some(pos) = version_str.find('-') {
            (&version_str[..pos], Some(&version_str[pos + 1..]))
        } else {
            (version_str, None)
        };

        let (pre_release, build) = if let Some(pre_and_build) = pre_and_build {
            if let Some(pos) = pre_and_build.find('+') {
                (
                    Some(pre_and_build[..pos].to_string()),
                    Some(pre_and_build[pos + 1..].to_string()),
                )
            } else {
                (Some(pre_and_build.to_string()), None)
            }
        } else if let Some(pos) = core_version.find('+') {
            let (_core, build_part) = core_version.split_at(pos);
            (None, Some(build_part[1..].to_string()))
        } else {
            (None, None)
        };

        let core_version = if let Some(pos) = core_version.find('+') {
            &core_version[..pos]
        } else {
            core_version
        };

        let parts: Vec<&str> = core_version.split('.').collect();

        // Ensure we have at least a major version
        if parts.is_empty() {
            return None;
        }

        let major = parts[0].parse().ok()?;
        let minor = parts.get(1).and_then(|v| v.parse().ok());
        let patch = parts.get(2).and_then(|v| v.parse().ok());

        Some(Version {
            major,
            minor,
            patch,
            pre_release,
            build,
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.major)?;

        if let Some(minor) = self.minor {
            write!(f, ".{minor}")?;

            if let Some(patch) = self.patch {
                write!(f, ".{patch}")?;
            }
        }

        if let Some(pre) = &self.pre_release {
            write!(f, "-{pre}")?;
        }

        if let Some(build) = &self.build {
            write!(f, "+{build}")?;
        }

        Ok(())
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare major, then minor (None = 0), then patch (None = 0)
        let major_ord = self.major.cmp(&other.major);
        if major_ord != std::cmp::Ordering::Equal {
            return major_ord;
        }

        let self_minor = self.minor.unwrap_or(0);
        let other_minor = other.minor.unwrap_or(0);
        let minor_ord = self_minor.cmp(&other_minor);
        if minor_ord != std::cmp::Ordering::Equal {
            return minor_ord;
        }

        let self_patch = self.patch.unwrap_or(0);
        let other_patch = other.patch.unwrap_or(0);
        let patch_ord = self_patch.cmp(&other_patch);
        if patch_ord != std::cmp::Ordering::Equal {
            return patch_ord;
        }

        // Pre-release comparison per semver spec:
        // No pre-release > has pre-release
        // Both have pre-release: compare identifiers per semver rules
        compare_pre_release(self.pre_release.as_ref(), other.pre_release.as_ref())
    }
}

/// Compare pre-release identifiers per semver spec (item 11).
/// - No pre-release > any pre-release
/// - Split by '.', compare each identifier:
///   - Numeric identifiers compared as integers
///   - Alphanumeric compared lexicographically
///   - Numeric < alphanumeric
///   - Shorter list is less (if all preceding are equal)
fn compare_pre_release(a: Option<&String>, b: Option<&String>) -> std::cmp::Ordering {
    match (a, b) {
        (None, None) => std::cmp::Ordering::Equal,
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(a), Some(b)) => compare_pre_release_identifiers(a, b),
    }
}

fn compare_pre_release_identifiers(a: &str, b: &str) -> std::cmp::Ordering {
    let a_parts: Vec<&str> = a.split('.').collect();
    let b_parts: Vec<&str> = b.split('.').collect();

    for (a_id, b_id) in a_parts.iter().zip(b_parts.iter()) {
        let ord = match (a_id.parse::<u64>(), b_id.parse::<u64>()) {
            (Ok(a_num), Ok(b_num)) => a_num.cmp(&b_num),
            (Ok(_), Err(_)) => std::cmp::Ordering::Less, // numeric < alphanumeric
            (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
            (Err(_), Err(_)) => a_id.cmp(b_id),
        };
        if ord != std::cmp::Ordering::Equal {
            return ord;
        }
    }

    a_parts.len().cmp(&b_parts.len())
}
