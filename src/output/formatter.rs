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

    // Check if we have multiple sources (workspace mode)
    let has_multiple_sources = filtered_results
        .iter()
        .map(|dep| &dep.source)
        .collect::<std::collections::HashSet<_>>()
        .len()
        > 1;

    // Calculate dynamic column widths
    let mut max_name_width = "Dependency".len();
    let mut max_current_width = "Current Version".len();
    let mut max_latest_width = "Latest Version".len();
    let mut max_status_width = "Status".len();
    let mut max_source_width = if has_multiple_sources {
        "Source".len()
    } else {
        0
    };

    // Prepare data with status strings to calculate accurate widths
    let display_data: Vec<_> = filtered_results
        .iter()
        .map(|dep| {
            let name_with_type = format!("{}{}", dep.name, dep.dep_type);
            let latest_display = dep.latest_version.as_deref().unwrap_or("N/A");
            let status = match &dep.latest_version {
                Some(latest) => {
                    if dep.is_outdated() {
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

            (
                name_with_type,
                &dep.current_version,
                latest_display,
                &dep.source,
                status,
            )
        })
        .collect();

    // Calculate maximum widths
    for (name, current, latest, source, status) in &display_data {
        max_name_width = max_name_width.max(name.len());
        max_current_width = max_current_width.max(current.len());
        max_latest_width = max_latest_width.max(latest.len());
        if has_multiple_sources {
            max_source_width = max_source_width.max(source.len());
        }
        max_status_width = max_status_width.max(status.len());
    }

    // Add padding to each column
    let name_width = max_name_width + 2;
    let current_width = max_current_width + 2;
    let latest_width = max_latest_width + 2;
    let source_width = if has_multiple_sources {
        max_source_width + 2
    } else {
        0
    };

    println!("\n📊 Dependency Check Results:");

    if has_multiple_sources {
        println!(
            "{:<name_width$} {:<current_width$} {:<latest_width$} {:<source_width$} Status",
            "Dependency",
            "Current Version",
            "Latest Version",
            "Source",
            name_width = name_width,
            current_width = current_width,
            latest_width = latest_width,
            source_width = source_width,
        );
    } else {
        println!(
            "{:<name_width$} {:<current_width$} {:<latest_width$} Status",
            "Dependency",
            "Current Version",
            "Latest Version",
            name_width = name_width,
            current_width = current_width,
            latest_width = latest_width,
        );
    }

    let mut outdated_count = 0;

    for (name, current, latest, source, status) in &display_data {
        if status.contains("Outdated") {
            outdated_count += 1;
        }

        if has_multiple_sources {
            println!(
                "{name:<name_width$} {current:<current_width$} {latest:<latest_width$} {source:<source_width$} {status}",
            );
        } else {
            println!(
                "{name:<name_width$} {current:<current_width$} {latest:<latest_width$} {status}",
            );
        }
    }

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
