use crate::{commands::models::DiffResult, ui::print_section};

pub fn print_status(diff: &DiffResult) {
    if diff.added.is_empty() && diff.modified.is_empty() && diff.removed.is_empty() {
        println!("Workspace is clean!");
        return;
    }
    println!(
        "{} added, {} modified, {} removed",
        diff.added.len(),
        diff.modified.len(),
        diff.removed.len()
    );
    print_section("Added", '+', &diff.added);
    print_section("Modified", '~', &diff.modified);
    print_section("Removed", '-', &diff.removed);
}
