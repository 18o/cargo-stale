use crate::version::core::Version;

pub fn is_version_outdated(current_req: &str, latest_version: &str) -> bool {
    let current_req = current_req.trim();
    if current_req == "*" {
        return false;
    }

    let current = match Version::parse(current_req) {
        Some(req) => req,
        None => return current_req != latest_version,
    };

    let latest = match Version::parse(latest_version) {
        Some(version) => version,
        None => return current_req != latest_version,
    };

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
        assert!(!is_version_outdated("=0.7.3", "0.7.3"));
    }
}
