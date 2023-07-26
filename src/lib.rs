pub mod changelog;
pub mod config;
pub mod git;
pub mod template;

use crate::changelog::Changelog;
use crate::config::Command;
use crate::git::command::GitLogCmd;
use crate::git::Git;
use crate::template::Template;
use std::error::Error;

pub fn run(config: config::Config) -> Result<(), Box<dyn Error>> {
    let template = Template::new(config.filename)?;

    let git_cmd = Box::new(GitLogCmd::new(config.git_path.to_string()));
    let git = Git::new(git_cmd);

    let changelog = Changelog::new(template, git);

    let output = changelog.produce()?;

    if let Command::Generate = config.command {
        println!("{}", output);
    }

    Ok(())
}
