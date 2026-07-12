use semver::{Version, VersionReq};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionStatus {
    Compatible,
    UpdateAvailable,
    PreReleaseAvailable,
    InvalidRequirement,
    InvalidLatestVersion,
    NotFound,
    Unavailable,
    UnsupportedSource,
}

impl VersionStatus {
    pub fn is_outdated(self) -> bool {
        matches!(self, Self::UpdateAvailable)
    }

    pub fn is_indeterminate(self) -> bool {
        matches!(
            self,
            Self::InvalidRequirement
                | Self::InvalidLatestVersion
                | Self::NotFound
                | Self::Unavailable
                | Self::UnsupportedSource
        )
    }
}

pub fn evaluate_requirement(requirement: &str, latest_version: &str) -> VersionStatus {
    let Ok(requirement) = VersionReq::parse(requirement) else {
        return VersionStatus::InvalidRequirement;
    };
    let Ok(latest) = Version::parse(latest_version) else {
        return VersionStatus::InvalidLatestVersion;
    };

    if requirement.matches(&latest) {
        VersionStatus::Compatible
    } else if !latest.pre.is_empty() {
        VersionStatus::PreReleaseAvailable
    } else {
        VersionStatus::UpdateAvailable
    }
}

#[cfg(test)]
mod tests {
    use super::{VersionStatus, evaluate_requirement};

    #[test]
    fn supports_cargo_requirement_operators() {
        assert_eq!(
            evaluate_requirement("^1.2", "1.9.0"),
            VersionStatus::Compatible
        );
        assert_eq!(
            evaluate_requirement("~1.2", "1.3.0"),
            VersionStatus::UpdateAvailable
        );
        assert_eq!(
            evaluate_requirement("=1.2.3", "1.2.3"),
            VersionStatus::Compatible
        );
        assert_eq!(
            evaluate_requirement(">=1.0", "2.0.0"),
            VersionStatus::Compatible
        );
    }

    #[test]
    fn reports_prereleases_without_calling_them_updates() {
        assert_eq!(
            evaluate_requirement("^1.0", "2.0.0-rc.1"),
            VersionStatus::PreReleaseAvailable
        );
    }
}
