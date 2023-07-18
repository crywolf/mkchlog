use mkchlog::config::Config;
use std::env;
use std::process;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });

    if let Err(err) = mkchlog::run(config) {
        eprintln!("{}", err);
        process::exit(1);
    }
}
