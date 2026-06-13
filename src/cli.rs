use crate::commands::Command;
use std::process;

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
        "save" => Command::Save {
            message: "Init".to_string(),
        },
        _ => Command::Init,
    }
}
