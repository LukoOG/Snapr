pub mod history;
pub mod init;
pub mod save;
pub mod diff;
pub enum Command {
    Init,
    Save { message: String },
    History,
    Diff (u32, u32)
}