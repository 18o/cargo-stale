#[derive(Debug)]
pub struct VersionRequirement {
    pub operator: String,
    pub version: String,
}

pub fn parse_version_requirement(req: &str) -> Option<VersionRequirement> {
    let req = req.trim();

    if let Some(version) = req.strip_prefix("^") {
        Some(VersionRequirement {
            operator: "^".to_string(),
            version: version.to_string(),
        })
    } else if let Some(version) = req.strip_prefix("~") {
        Some(VersionRequirement {
            operator: "~".to_string(),
            version: version.to_string(),
        })
    } else if let Some(version) = req.strip_prefix("=") {
        Some(VersionRequirement {
            operator: "=".to_string(),
            version: version.to_string(),
        })
    } else if let Some(version) = req.strip_prefix(">=") {
        Some(VersionRequirement {
            operator: ">=".to_string(),
            version: version.to_string(),
        })
    } else if let Some(version) = req.strip_prefix("<=") {
        Some(VersionRequirement {
            operator: "<=".to_string(),
            version: version.to_string(),
        })
    } else if let Some(version) = req.strip_prefix(">") {
        Some(VersionRequirement {
            operator: ">".to_string(),
            version: version.to_string(),
        })
    } else if let Some(version) = req.strip_prefix("<") {
        Some(VersionRequirement {
            operator: "<".to_string(),
            version: version.to_string(),
        })
    } else {
        Some(VersionRequirement {
            operator: "^".to_string(),
            version: req.to_string(),
        })
    }
}

pub fn parse_simple_version(version: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 3 {
        if let (Ok(major), Ok(minor), Ok(patch)) = (
            parts[0].parse::<u32>(),
            parts[1].parse::<u32>(),
            parts[2].parse::<u32>(),
        ) {
            return Some((major, minor, patch));
        }
    } else if parts.len() == 2 {
        if let (Ok(major), Ok(minor)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
            return Some((major, minor, 0));
        }
    } else if parts.len() == 1 {
        if let Ok(major) = parts[0].parse::<u32>() {
            return Some((major, 0, 0));
        }
    }
    None
}
