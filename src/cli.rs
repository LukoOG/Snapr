use std::cmp;
use clap::Parser;
use crate::{commands::{Command, models::RestoreOptions}, error::SnaprResult};

#[derive(Parser)]
#[command(name = "snapr")]
#[command(about = "A lightweight content-addressed snapshot tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

fn parse_snapshot_id(args: &[String], index: usize, name: &str) -> u32 {
    args.get(index)
        .expect(&format!("Provide argument for {name} id"))
        .parse::<u32>()
        .expect(&format!(
            "Provided argument for {name} id must be an integer"
        ))
}

pub fn parse_args(args: &[String]) -> SnaprResult<Command> {
    let length = args.len();
    if length < 2 {
        eprintln!("No arguements provided!");
        std::process::exit(1)
    };

    let arg = args[1].as_str();

    match arg {
        "init" => Ok(Command::Init),
        "history" => Ok(Command::History),
        "save" => {
            if let Some(message) = args.get(2) {
                return Ok(Command::Save {
                    message: message.clone(),
                });
            } else {
                return Err("Message not provided!".into());
            }
        }
        "diff" => {
            let id_1 = parse_snapshot_id(args, 2, "old");
            let id_2 = parse_snapshot_id(args, 3, "new");

            let old = cmp::min(id_1, id_2);
            let new = cmp::max(id_1, id_2);

            Ok(Command::Diff{ old, new })
        }
        "restore" => {
            let snapshot_id = match args.get(2) {
                Some(id) => id
                    .parse::<u32>()
                    .map_err(|_| "Snapshot id must be an integer")?,
                None => return Err("Provide snapshot id".into()),
            };

            let force = args.iter().any(|arg| arg == "--force");
            let dry_run = args.iter().any(|arg| arg == "--dry-run");

            Ok(Command::Restore(RestoreOptions {
                snapshot_id,
                force,
                dry_run,
            }))
        }
        "status" => Ok(Command::Status),
        "verify" => Ok(Command::Verify),
        _ => Err("Unknown Command!".into()),
    }
}
