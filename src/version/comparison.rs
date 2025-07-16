use super::parser::{parse_simple_version, parse_version_requirement};

pub fn is_version_outdated(current_req: &str, latest_version: &str) -> bool {
    let current_req = current_req.trim();

    if let Some(parsed_req) = parse_version_requirement(current_req) {
        if let (Some(current_version), Some(latest_parsed)) = (
            parse_simple_version(&parsed_req.version),
            parse_simple_version(latest_version),
        ) {
            match parsed_req.operator.as_str() {
                "^" | "" => {
                    if current_version.0 == 0 {
                        current_version.1 != latest_parsed.1 || current_version.0 != latest_parsed.0
                    } else {
                        current_version.0 != latest_parsed.0
                    }
                }
                "~" => current_version.0 != latest_parsed.0 || current_version.1 != latest_parsed.1,
                "=" => parsed_req.version != latest_version,
                ">=" | ">" | "<=" | "<" => false,
                _ => parsed_req.version != latest_version,
            }
        } else {
            parsed_req.version != latest_version
        }
    } else {
        current_req != latest_version
    }
}
