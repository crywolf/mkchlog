use std::env;

pub struct Config {
    pub command: Command,
    pub filename: String,
    pub git_path: String,
}

pub enum Command {
    Check,
    Generate,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Self, String> {
        args.next(); // skip binary name

        let command = match args.next() {
            Some(arg) => match arg.as_str() {
                "check" => Command::Check,
                "gen" => Command::Generate,
                _ => return Err(format!("Unknown command '{}'", arg)),
            },
            None => return Err("Missing 'command' argument (check | gen)".into()),
        };

        let filename = match args.next() {
            Some(filename) => filename,
            None => ".mkchlog.yml".to_owned(),
        };

        let git_path = match args.next() {
            Some(git) => git,
            None => "./".to_owned(),
        };

        Ok(Self {
            command,
            filename,
            git_path,
        })
    }
}
