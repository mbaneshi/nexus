//! Output formatting utilities.

use serde::Serialize;

/// Print a value as JSON to stdout.
pub fn print_json<T: Serialize>(value: &T) -> crate::Result<()> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| crate::NexusError::Serialization(e.to_string()))?;
    println!("{json}");
    Ok(())
}

/// Print a value as a simple table row.
pub fn print_row(label: &str, value: &str) {
    println!("{:<20} {}", label, value);
}

/// Format bytes into human-readable size.
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_size_units() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
        assert_eq!(format_size(1073741824), "1.0 GB");
    }
}
