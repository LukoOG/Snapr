use std::{fs, error::Error};
use crate::{config::{load_config, save_config}, models::{FileEntry, Snapshot, StoreReport}};

pub fn handle_save(snapshots: &mut Vec<Snapshot>, message: String, entries: Vec<FileEntry>) -> Result<(), Box<dyn Error>>{
    let mut config = load_config()?;
    let next_id =  snapshots.iter().map(|s| s.id).max().unwrap_or(0) + 1;
    let new_snapshot = Snapshot {
        id: next_id,
        message,
        files: entries,
    };
    snapshots.push(new_snapshot);
    let json = serde_json::to_string_pretty(snapshots)?;
    fs::write(".snapr/snapshots.json", json)?;

    //config
    config.update_current_snapshot();
    save_config(&config)?;
    Ok(())
}

pub fn print_save_report(
    snapshot_id: u32,
    message: &str,
    report: &StoreReport,
) {
    let compression = if report.original_bytes > 0 {
        100.0 * (1.0 - report.new_storage_bytes as f64 / report.original_bytes as f64)
    } else {
        0.0
    };
    println!("Created Snapshot {}\n", snapshot_id);

    println!("Message:");
    println!("\"{}\"\n", message);

    println!("{} files scanned", report.total_files);
    println!("{} new objects stored", report.new_objects);
    println!("{} objects reused\n", report.reused_objects);

    println!(
        "Current Workspace Size:   {}",
        format_bytes(report.original_bytes)
    );

    println!(
        "New Storage Used:   {}",
        format_bytes(report.new_storage_bytes)
    );

    println!(
        "Compression:   {:.1}%",
        compression
    );
}

#[inline]
fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1 << 10;
    const MB: usize = 1 << 20;
    const GB: usize = 1 << 30;

    if bytes < KB {
        format!("{bytes} B")
    } else if bytes < MB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    }
}