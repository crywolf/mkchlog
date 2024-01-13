use mkchlog::config::Config;
use std::process;

fn main() {
    let config = Config::new().unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        process::exit(1);
    });

    if let Err(err) = mkchlog::run(config) {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}
