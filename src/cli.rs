use std::{cmp};

use crate::commands::Command;

fn parse_snapshot_id(args: &[String], index: usize, name: &str) -> u32 {
    args.get(index)
        .expect(&format!("Provide argument for {name} id"))
        .parse::<u32>()
        .expect(&format!(
            "Provided argument for {name} id must be an integer"
        ))
}

pub fn parse_args(args: &[String]) -> Command {
    let length = args.len();
    if length < 2 {
        println!("No arguements provided!");
        std::process::exit(1)
    };

    let arg = args[1].as_str();

    match arg {
        "init" => Command::Init,
        "history" => Command::History,
        "save" => {
            if let Some(message) = args.get(2) {
                return Command::Save {
                    message: message.clone(),
                };
            } else {
                eprintln!("Message not provided!");
                std::process::exit(1)
            }
        }
        "diff" => {
            let id_1 = parse_snapshot_id(args, 2, "old");
            let id_2 = parse_snapshot_id(args, 3, "new");

            let old_id = cmp::min(id_1, id_2);
            let new_id = cmp::max(id_1, id_2);

            Command::Diff(old_id, new_id)
        }
        _ => Command::Init,
    }
}
