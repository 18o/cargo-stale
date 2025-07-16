use crate::cli::Cli;
use crate::types::Dependency;

pub fn print_results(results: &[Dependency], cli: &Cli) {
    let filtered_results: Vec<_> = if cli.outdated_only {
        results.iter().filter(|dep| dep.is_outdated()).collect()
    } else {
        results.iter().collect()
    };

    if filtered_results.is_empty() {
        if cli.outdated_only {
            println!("🎉 No outdated dependencies found!");
        } else {
            println!("❌ No dependencies found");
        }
        return;
    }

    println!("\n📊 Dependency Check Results:");
    println!("{:-<110}", "");
    println!(
        "{:<35} {:<20} {:<20} {:<16} {:<20}",
        "Dependency", "Current Version", "Latest Version", "Status", "Source"
    );
    println!("{:-<110}", "");

    let mut outdated_count = 0;

    for dep in &filtered_results {
        let status = match &dep.latest_version {
            Some(latest) => {
                if dep.is_outdated() {
                    outdated_count += 1;
                    if is_prerelease_version(latest) {
                        "🟡 Outdated (Pre)"
                    } else {
                        "🔴 Outdated"
                    }
                } else if is_prerelease_version(latest) {
                    "🟢 Latest (Pre)"
                } else {
                    "✅ Latest"
                }
            }
            None => "❓ Unknown",
        };

        let latest_display = dep.latest_version.as_deref().unwrap_or("N/A");
        let name_with_type = format!("{}{}", dep.name, dep.dep_type);

        println!(
            "{:<35} {:<20} {:<20} {:<16} {:<20}",
            name_with_type, dep.current_version, latest_display, status, dep.source
        );
    }

    println!("{:-<110}", "");

    if outdated_count > 0 {
        println!("⚠️  Found {outdated_count} outdated dependencies");
        if cli.verbose {
            println!("💡 Use 'cargo update <crate_name>' to update specific dependencies");
        }
    } else if !cli.outdated_only {
        println!("🎉 All dependencies are up to date!");
    }
}

/// Check if a version string contains pre-release identifiers
fn is_prerelease_version(version: &str) -> bool {
    version.contains('-')
        && (version.contains("alpha") ||
        version.contains("beta") ||
        version.contains("rc") ||
        version.contains("pre") ||
        version.contains("dev") ||
        // Also catch numbered pre-releases like "1.0.0-1"
        version.split('-').nth(1).is_some_and(|part| {
            part.chars().next().is_some_and(|c| c.is_ascii_digit())
        }))
}
