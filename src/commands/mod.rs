pub mod history;
pub mod init;
pub mod save;
pub mod diff;
pub mod restore;
pub mod status;
pub mod models;
mod helpers;
pub enum Command {
    Init,
    Save { message: String },
    History,
    Diff (u32, u32),
    Restore(models::RestoreOptions),
    Status,
}