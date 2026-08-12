use super::format_bytes;
use crate::models::{VerifyReport};

pub fn print_verify_report(report: &VerifyReport) {
    println!("✓ Repository verified");

    println!("\n────────────────────────────────────────");

    println!("\nRepository");
    println!("  Snapshots checked : {}", report.snapshots_checked);
    println!("  Files checked     : {}", report.files_checked);

    println!("\nChunks");
    println!("  Referenced        : {}", report.chunks_referenced);
    println!("  Verified          : {}", report.chunks_verified);

    println!("\nData");
    println!("  Bytes verified    : {}", format_bytes(report.bytes_verified as usize));

    if report.issues.is_empty() {
        println!("\n✓ No integrity issues found.");
    } else {
        println!("\n⚠ {} integrity issues found.", report.issues.len());
    }
}
