use super::format_bytes;
use crate::models::RestoreReport;

pub fn print_restore_report(report: &RestoreReport) {
    println!("✓ Restored snapshot {}", report.snapshot_id);

    println!("\n────────────────────────────────────────");

    println!("\nWorkspace");
    println!("Restored: {} files", report.restored_files);
    println!("Removed: {} files", report.removed_files);
    println!("Skipped: {} files", report.skipped_files);

    println!("\n Data");
    println!("Restored bytes: {}", format_bytes(report.restored_bytes as f64));

    println!("\nWorkspace updated 📸.");
}

pub fn print_restore_dry_run_report(report: &RestoreReport) {
    println!("Dry run: Snapshot {} would be restored", report.snapshot_id);

    println!("\n────────────────────────────────────────");

    println!("\nWorkspace Changes");
    println!("{} files would be restored", report.restored_files);
    println!("{} files would be removed", report.removed_files);
    println!("{} files would be skipped", report.skipped_files);

    println!("\n Data");
    println!("Restored Storage: {}", format_bytes((report.restored_bytes as usize) as f64));

    println!("\nNo changes were made to the workspace.");
}