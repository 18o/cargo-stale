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
        // Compare major version first
        match self.major.cmp(&other.major) {
            std::cmp::Ordering::Equal => {
                // If either version has no minor, they are equal at major level
                match (&self.minor, &other.minor) {
                    (None, None) => {
                        // Both have no minor, handle pre-release comparison
                        match (&self.pre_release, &other.pre_release) {
                            (None, None) => std::cmp::Ordering::Equal,
                            (Some(_), None) => std::cmp::Ordering::Less, // Pre-release < normal
                            (None, Some(_)) => std::cmp::Ordering::Greater, // Normal > pre-release
                            (Some(a), Some(b)) => a.cmp(b), // Compare pre-release strings
                        }
                    }
                    (None, Some(_)) | (Some(_), None) => std::cmp::Ordering::Equal,
                    (Some(self_minor), Some(other_minor)) => {
                        // Compare minor versions
                        match self_minor.cmp(other_minor) {
                            std::cmp::Ordering::Equal => {
                                // If either version has no patch, they are equal at minor level
                                match (&self.patch, &other.patch) {
                                    (None, None) => {
                                        // Both have no patch, handle pre-release comparison
                                        match (&self.pre_release, &other.pre_release) {
                                            (None, None) => std::cmp::Ordering::Equal,
                                            (Some(_), None) => std::cmp::Ordering::Less,
                                            (None, Some(_)) => std::cmp::Ordering::Greater,
                                            (Some(a), Some(b)) => a.cmp(b),
                                        }
                                    }
                                    (None, Some(_)) | (Some(_), None) => std::cmp::Ordering::Equal,
                                    (Some(self_patch), Some(other_patch)) => {
                                        // Compare patch versions
                                        match self_patch.cmp(other_patch) {
                                            std::cmp::Ordering::Equal => {
                                                // Handle pre-release comparison
                                                match (&self.pre_release, &other.pre_release) {
                                                    (None, None) => std::cmp::Ordering::Equal,
                                                    (Some(_), None) => std::cmp::Ordering::Less,
                                                    (None, Some(_)) => std::cmp::Ordering::Greater,
                                                    (Some(a), Some(b)) => a.cmp(b),
                                                }
                                            }
                                            patch_ordering => patch_ordering,
                                        }
                                    }
                                }
                            }
                            minor_ordering => minor_ordering,
                        }
                    }
                }
            }
            major_ordering => major_ordering,
        }
    }
}
