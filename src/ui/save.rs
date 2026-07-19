use super::format_bytes;
use crate::models::WorkspaceStoreReport;

pub fn print_save_report(snapshot_id: u32, message: &str, report: &WorkspaceStoreReport) {
    println!("✓ Snapshot {} created", snapshot_id);
    println!("  \"{}\"", message);

    println!("\n────────────────────────────────────────");

    println!("\nWorkspace");
    println!("  Files processed : {}", report.total_files);
    println!("  Chunks processed: {}", report.total_chunks);
    println!(
        "  Workspace size  : {}",
        format_bytes(report.original_bytes)
    );

    println!("\nObject Store");
    println!("  New chunks      : {}", report.new_chunks);
    println!("  Reused chunks   : {}", report.reused_chunks);
    println!(
        "  Additional Storage : {}",
        format_bytes(report.new_storage_bytes)
    );

    println!("\nEfficiency");
    println!(
        "  Chunk reuse     : {:.2}%",
        report.deduplication_ratio() * 100.0
    );
    println!("  Compression saved : {:.2}%", report.compression_ratio());

    println!("\nSnapshot complete 📸.");
}
