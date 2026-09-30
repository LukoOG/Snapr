use clap::Args;

type FilePath = String;

#[derive(Debug)]
#[derive(Default)]
pub struct DiffResult {
    pub added: Vec<FilePath>,
    pub modified: Vec<FilePath>,
    pub removed: Vec<FilePath>,
}


#[derive(Args)]
pub struct RestoreOptions {
    pub snapshot_id: u32,

    #[arg(long)]
    pub force: bool,

    #[arg(long)]
    pub dry_run: bool,
}