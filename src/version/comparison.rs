use crate::version::core::Version;

/// Check if a dependency version requirement is outdated compared to the latest version.
///
/// Uses prefix matching: if Cargo.toml specifies a partial version like "1" or "1.2",
/// any version matching that prefix is considered compatible (not outdated).
/// - "1" matches all 1.x.x → not outdated
/// - "1.2" matches all 1.2.x → not outdated
/// - "1.2.3" only matches 1.2.3 exactly → outdated if latest is higher
pub fn is_version_outdated(current_req: &str, latest_version: &str) -> bool {
    let current_req = current_req.trim();
    if current_req == "*" {
        return false;
    }

    let Some(current) = Version::parse(current_req) else {
        return current_req != latest_version;
    };

    let Some(latest) = Version::parse(latest_version) else {
        return current_req != latest_version;
    };

    // Prefix matching: if current has no minor, only compare major
    // If current has no patch, only compare major.minor
    if current.minor.is_none() {
        // "1" → any 1.x.x is compatible
        return current.major != latest.major;
    }

    if current.patch.is_none() {
        // "1.2" → any 1.2.x is compatible
        return current.major != latest.major
            || current.minor != latest.minor;
    }

    // Full version "1.2.3" → exact comparison
    current < latest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_version_outdated() {
        assert!(is_version_outdated("0.7.1", "0.7.2"));
        assert!(!is_version_outdated("0.7.1", "0.7.0"));
        assert!(!is_version_outdated("0.7.1", "0.5.92"));
        assert!(is_version_outdated("4.0.0-rc.3", "4.0.0"));
        assert!(!is_version_outdated("4.0.0", "4.0.0+build.123"));
    }

    #[test]
    fn test_prefix_version_matching() {
        // "1" → any 1.x.x is compatible, not outdated
        assert!(!is_version_outdated("1", "1.51.1"));
        assert!(!is_version_outdated("1", "1.0.0"));
        // "1" vs "2.x.x" → major changed, outdated
        assert!(is_version_outdated("1", "2.0.0"));

        // "1.2" → any 1.2.x is compatible, not outdated
        assert!(!is_version_outdated("1.2", "1.2.3"));
        assert!(!is_version_outdated("1.2", "1.2.0"));
        // "1.2" vs "1.3.x" → minor changed, outdated
        assert!(is_version_outdated("1.2", "1.3.0"));
        // "1.2" vs "2.0.x" → major changed, outdated
        assert!(is_version_outdated("1.2", "2.0.0"));

        // "1.0" equals "1.0.0"
        assert!(!is_version_outdated("1.0", "1.0.0"));
        // "1" equals "1.0"
        assert!(!is_version_outdated("1", "1.0"));
    }

    #[test]
    fn test_prerelease_semver_comparison() {
        // Numeric pre-release: alpha.10 > alpha.2
        assert!(is_version_outdated("1.0.0-alpha.2", "1.0.0-alpha.10"));
        // Pre-release < release
        assert!(is_version_outdated("1.0.0-alpha.1", "1.0.0"));
        // Numeric < alphanumeric
        assert!(is_version_outdated("1.0.0-1", "1.0.0-beta"));
    }
}
