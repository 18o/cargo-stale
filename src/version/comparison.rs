use semver::{Comparator, Op, Version, VersionReq};

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

    if requirement.matches(&latest) || has_newer_prerelease_floor(&requirement, &latest) {
        VersionStatus::Compatible
    } else if !latest.pre.is_empty() {
        VersionStatus::PreReleaseAvailable
    } else {
        VersionStatus::UpdateAvailable
    }
}

fn has_newer_prerelease_floor(requirement: &VersionReq, latest: &Version) -> bool {
    requirement.comparators.iter().any(|comparator| {
        is_lower_bound(comparator)
            && !comparator.pre.is_empty()
            && comparator_version(comparator) > *latest
    })
}

fn is_lower_bound(comparator: &Comparator) -> bool {
    matches!(
        comparator.op,
        Op::Exact | Op::Greater | Op::GreaterEq | Op::Tilde | Op::Caret | Op::Wildcard
    )
}

fn comparator_version(comparator: &Comparator) -> Version {
    let mut version = Version::new(
        comparator.major,
        comparator.minor.unwrap_or(0),
        comparator.patch.unwrap_or(0),
    );
    version.pre = comparator.pre.clone();
    version
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

    #[test]
    fn does_not_offer_an_older_stable_version_for_a_newer_prerelease_requirement() {
        assert_eq!(
            evaluate_requirement("^6.0.0-rc.29", "5.3.0"),
            VersionStatus::Compatible
        );
        assert_eq!(
            evaluate_requirement("^3.0.0-rc.12", "2.2.6"),
            VersionStatus::Compatible
        );
        assert_eq!(
            evaluate_requirement("^6.0.0-rc.29", "7.0.0"),
            VersionStatus::UpdateAvailable
        );
    }
}
