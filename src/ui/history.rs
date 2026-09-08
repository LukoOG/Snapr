use crate::models::Snapshot;
use super::{format_bytes, format_timestamp};

pub fn print_history(snapshots: &[Snapshot], current_snapshot_id: u32) {
    println!("Snapr History");
    println!("────────────────────────────────────────");

    for snapshot in snapshots.iter().rev() {
        print_snapshot(snapshot, current_snapshot_id);
    }

    println!("────────────────────────────────────────");
    println!("{} snapshots", snapshots.len());
}

fn print_snapshot(snapshot: &Snapshot, current_snapshot_id: u32) {
    let marker = if snapshot.id == current_snapshot_id {
        "*"
    } else {
        " "
    };

    println!(
        "{} Snapshot {}   {}",
        marker,
        snapshot.id,
        format_timestamp(snapshot.created_at)
    );

    println!("  \"{}\"", snapshot.message);

    println!();

    println!(
        "  {} files · {} chunks · {}",
        snapshot.stats.file_count,
        snapshot.stats.chunk_count,
        format_bytes(snapshot.stats.workspace_bytes as usize)
    );

    println!(
        "  +{} new chunks · +{} storage",
        snapshot.stats.new_chunks,
        format_bytes(snapshot.stats.new_storage_bytes as usize)
    );

    println!(
        "  Total storage · {}",
        format_bytes(snapshot.stats.total_storage_bytes as usize)
    );

    println!();
}
