use crate::cli::Cli;
use crate::types::{Dependency, VersionLookup};
use crate::version::comparison::VersionStatus;

type DisplayData<'a> = (String, &'a str, &'a str, &'a str, &'static str);

#[derive(Debug)]
struct ColumnWidths {
    name: usize,
    current: usize,
    latest: usize,
    source: usize,
}

pub fn print_results(results: &[Dependency], cli: &Cli) {
    let filtered_results = filter_results(results, cli);

    if filtered_results.is_empty() {
        print_empty_results_message(cli);
        return;
    }

    let has_multiple_sources = check_multiple_sources(&filtered_results);
    let display_data = prepare_display_data(&filtered_results);
    let column_widths = calculate_column_widths(&display_data, has_multiple_sources);

    print_header(&column_widths, has_multiple_sources);
    let outdated_count = print_dependency_rows(&display_data, &column_widths, has_multiple_sources);
    let has_indeterminate = filtered_results
        .iter()
        .any(|dependency| dependency.status().is_indeterminate());
    print_summary(outdated_count, has_indeterminate, cli);
}

fn filter_results<'a>(results: &'a [Dependency], cli: &Cli) -> Vec<&'a Dependency> {
    if cli.output_filter().is_outdated_only() {
        results.iter().filter(|dep| dep.is_outdated()).collect()
    } else {
        results.iter().collect()
    }
}

fn print_empty_results_message(cli: &Cli) {
    if cli.output_filter().is_outdated_only() {
        println!("🎉 No outdated dependencies found!");
    } else {
        println!("❌ No dependencies found");
    }
}

fn check_multiple_sources(filtered_results: &[&Dependency]) -> bool {
    filtered_results
        .iter()
        .map(|dep| &dep.source)
        .collect::<std::collections::HashSet<_>>()
        .len()
        > 1
}

fn prepare_display_data<'a>(filtered_results: &[&'a Dependency]) -> Vec<DisplayData<'a>> {
    filtered_results
        .iter()
        .map(|dep| {
            let name_with_type = format!("{}{}", dep.name, dep.dep_type);
            let latest_display = match &dep.lookup {
                VersionLookup::Found(version) => version.as_str(),
                VersionLookup::NotFound => "Not found",
                VersionLookup::Unavailable => "Unavailable",
                VersionLookup::UnsupportedSource => "Unsupported",
            };
            let status = get_status_text(dep);

            (
                name_with_type,
                dep.current_version.as_str(),
                latest_display,
                dep.source.as_str(),
                status,
            )
        })
        .collect()
}

fn get_status_text(dep: &Dependency) -> &'static str {
    match dep.status() {
        VersionStatus::Compatible => "✅ Compatible",
        VersionStatus::UpdateAvailable => "🔴 Update available",
        VersionStatus::PreReleaseAvailable => "🟡 Pre-release available",
        VersionStatus::InvalidRequirement => "❓ Invalid requirement",
        VersionStatus::InvalidLatestVersion => "❓ Invalid latest version",
        VersionStatus::NotFound => "❓ Not found",
        VersionStatus::Unavailable => "⚠️  Unavailable",
        VersionStatus::UnsupportedSource => "⚪ Unsupported source",
    }
}

fn calculate_column_widths(
    display_data: &[DisplayData<'_>],
    has_multiple_sources: bool,
) -> ColumnWidths {
    let mut max_name_width = "Dependency".len();
    let mut max_current_width = "Current Version".len();
    let mut max_latest_width = "Latest Version".len();
    let mut max_source_width = if has_multiple_sources {
        "Source".len()
    } else {
        0
    };

    for (name, current, latest, source, _status) in display_data {
        max_name_width = max_name_width.max(name.len());
        max_current_width = max_current_width.max(current.len());
        max_latest_width = max_latest_width.max(latest.len());
        if has_multiple_sources {
            max_source_width = max_source_width.max(source.len());
        }
    }

    ColumnWidths {
        name: max_name_width + 2,
        current: max_current_width + 2,
        latest: max_latest_width + 2,
        source: if has_multiple_sources {
            max_source_width + 2
        } else {
            0
        },
    }
}

fn print_header(widths: &ColumnWidths, has_multiple_sources: bool) {
    println!("\n📊 Dependency Check Results:");

    if has_multiple_sources {
        println!(
            "{:<name_width$} {:<current_width$} {:<latest_width$} {:<source_width$} Status",
            "Dependency",
            "Current Version",
            "Latest Version",
            "Source",
            name_width = widths.name,
            current_width = widths.current,
            latest_width = widths.latest,
            source_width = widths.source,
        );
    } else {
        println!(
            "{:<name_width$} {:<current_width$} {:<latest_width$} Status",
            "Dependency",
            "Current Version",
            "Latest Version",
            name_width = widths.name,
            current_width = widths.current,
            latest_width = widths.latest,
        );
    }
}

fn print_dependency_rows(
    display_data: &[DisplayData<'_>],
    widths: &ColumnWidths,
    has_multiple_sources: bool,
) -> usize {
    let mut outdated_count = 0;

    for (name, current, latest, source, status) in display_data {
        if status.contains("Update available") {
            outdated_count += 1;
        }

        if has_multiple_sources {
            println!(
                "{name:<name_width$} {current:<current_width$} {latest:<latest_width$} {source:<source_width$} {status}",
                name_width = widths.name,
                current_width = widths.current,
                latest_width = widths.latest,
                source_width = widths.source,
            );
        } else {
            println!(
                "{name:<name_width$} {current:<current_width$} {latest:<latest_width$} {status}",
                name_width = widths.name,
                current_width = widths.current,
                latest_width = widths.latest,
            );
        }
    }

    outdated_count
}

fn print_summary(outdated_count: usize, has_indeterminate: bool, cli: &Cli) {
    if outdated_count > 0 {
        println!("⚠️  Found {outdated_count} outdated dependencies");
        if cli.output_verbosity().is_verbose() {
            println!("💡 Use 'cargo update <crate_name>' to update specific dependencies");
        }
    } else if has_indeterminate {
        println!("⚠️  Results are incomplete; some dependencies could not be evaluated.");
    } else if !cli.output_filter().is_outdated_only() {
        println!("🎉 All dependencies are up to date!");
    }
}
