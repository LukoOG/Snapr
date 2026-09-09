use super::format_bytes;
use crate::models::VerifyReport;

pub fn print_verify_report(report: &VerifyReport) {
    println!("✓ Repository verified");

    println!("\n────────────────────────────────────────");

    println!("\nSnapshots");
    println!("  Checked           : {}", report.snapshots_checked);

    println!("\nFile versions");
    println!(
        "  Checked           : {}",
        report.unique_file_versions_checked
    );

    println!("\nChunks");
    println!("  Unique            : {}", report.total_chunks);
    println!("  Referenced        : {}", report.chunks_referenced);
    println!("  Verified          : {}", report.chunks_verified);
    println!("  Issues            : {}", report.issues.len());

    println!("\nData");
    println!(
        "  Content verified  : {}",
        format_bytes(report.bytes_verified as f64)
    );

    if report.issues.is_empty() {
        println!("\n✓ No integrity issues found.");
    } else {
        println!("\n⚠ {} integrity issues found.", report.issues.len());
    }
}
