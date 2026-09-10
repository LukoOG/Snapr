use super::format_bytes;
use crate::models::RestoreReport;

pub fn print_restore_report(report: &RestoreReport) {
    println!("✓ Restored snapshot {}", report.snapshot_id);

    println!("\n────────────────────────────────────────");

    println!("\nWorkspace");
    println!("  Restored: {} files", report.restored_files);
    println!("  Removed: {} files", report.removed_files);
    println!("  Skipped: {} files", report.skipped_files);

    println!("\n Data");
    println!("  Restored bytes: {}", format_bytes(report.restored_bytes as f64));

    println!("\n✓ Workspace restored to Snapshot {}", report.snapshot_id);
}

pub fn print_restore_dry_run_report(report: &RestoreReport) {
    println!("Restore Preview — Snapshot {}", report.snapshot_id);

    println!("\n────────────────────────────────────────");

    println!("\nWorkspace Changes");
    println!("  Would restore : {} files", report.restored_files);
    println!("  Would remove: {} files", report.removed_files);
    println!("  Unchanged: {} files", report.skipped_files);

    println!("\nNo changes were made to the workspace.");
}