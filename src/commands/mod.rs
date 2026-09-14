use clap::Subcommand;

use crate::commands::models::RestoreOptions;

pub mod history;
pub mod init;
pub mod save;
pub mod diff;
pub mod restore;
pub mod status;
pub mod models;
pub mod verify;
mod helpers;


#[derive(Subcommand)]
pub enum Command {
    Init,
    Save { message: String },
    History,
    Diff { old: u32, new: u32 },
    Restore (RestoreOptions),
    Status,
    Verify,
}