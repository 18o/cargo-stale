use crate::version::core::Version;

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
}
