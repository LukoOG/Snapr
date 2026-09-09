use crate::{
    commands::models::DiffResult,
    ui::{change_summary, print_section},
};

pub fn print_diff(diff: &DiffResult, old: u32, new: u32) {
    println!("Comparing Snapshot {} → {}", old, new);
    if diff.added.is_empty() && diff.modified.is_empty() && diff.removed.is_empty() {
        println!("Workspace is clean!");
        return;
    }
    println!("{}", change_summary(&diff));
    print_section("Added", '+', &diff.added);
    print_section("Modified", '~', &diff.modified);
    print_section("Removed", '-', &diff.removed);
}
