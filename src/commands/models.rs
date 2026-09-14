use clap::Args;

type FilePath = String;

#[derive(Debug)]
pub struct DiffResult {
    pub added: Vec<FilePath>,
    pub modified: Vec<FilePath>,
    pub removed: Vec<FilePath>,
}

impl Default for DiffResult {
    fn default() -> Self {
        DiffResult {
            added: Vec::new(),
            modified: Vec::new(),
            removed: Vec::new(),
        }
    }
}

#[derive(Args)]
pub struct RestoreOptions {
    pub snapshot_id: u32,

    #[arg(long)]
    pub force: bool,

    #[arg(long)]
    pub dry_run: bool,
}