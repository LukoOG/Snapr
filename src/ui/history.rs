use crate::models::Snapshot;
use super::{format_bytes, format_timestamp};

pub fn print_history(snapshots: &[Snapshot], current_snapshot_id: u32) {
    println!("Snapshot history ({})", snapshot_count(snapshots.len()));
    println!("────────────────────────────────────────────────────────────────");

    for snapshot in snapshots.iter().rev() {
        print_snapshot(snapshot, current_snapshot_id);
    }

    println!("────────────────────────────────────────────────────────────────");
    let total_storage_bytes = snapshots
        .iter()
        .find(|snapshot| snapshot.id == current_snapshot_id)
        .or_else(|| snapshots.last())
        .map_or(0, |snapshot| snapshot.stats.total_storage_bytes);
    println!("Storage: {}", format_bytes(total_storage_bytes as f64));
    println!();
    println!("- current snapshot");
}

fn snapshot_count(count: usize) -> String {
    match count {
        1 => "1 snapshot".to_owned(),
        _ => format!("{count} snapshots"),
    }
}

fn print_snapshot(snapshot: &Snapshot, current_snapshot_id: u32) {
    let marker = if snapshot.id == current_snapshot_id {
        "-"
    } else {
        " "
    };

    println!(
        "{} #{}  {}",
        marker,
        snapshot.id,
        format_timestamp(snapshot.created_at),
    );
    println!("  {}", snapshot.message);
    println!(
        "  {} files · {} processed",
        snapshot.stats.file_count,
        format_bytes(snapshot.stats.workspace_bytes as f64),
    );
    println!(
        "  +{} chunks · +{} storage",
        snapshot.stats.new_chunks,
        format_bytes(snapshot.stats.new_storage_bytes as f64),
    );
    println!();
}
