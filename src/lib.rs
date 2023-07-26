pub mod changelog;
pub mod config;
pub mod git;
pub mod template;

use crate::config::Command;
use changelog::Changelog;
use git::Git;
use std::error::Error;
use template::Template;

pub fn run(config: config::Config) -> Result<(), Box<dyn Error>> {
    let template = Template::new(config.filename)?;

    let git = Git::new(config.git_path.to_string());

    let changelog = Changelog::new(template, git);

    let output = changelog.produce()?;

    if let Command::Generate = config.command {
        println!("{}", output);
    }

    Ok(())
}
