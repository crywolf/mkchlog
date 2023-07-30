pub mod changelog;
pub mod config;
pub mod git;
pub mod template;

use crate::changelog::Changelog;
use crate::config::Command;
use crate::git::command::GitLogCmd;
use crate::git::Git;
use crate::template::Template;

pub fn run(config: config::Config) -> Result<(), Box<dyn std::error::Error>> {
    let template = Template::new(config.file_path)?;

    let git_cmd = Box::new(GitLogCmd::new(config.git_path, config.commit_id));
    let git = Git::new(git_cmd);

    let changelog = Changelog::new(template, git);

    let output = changelog.produce()?;

    if let Command::Generate = config.command {
        println!("{}", output);
    }

    Ok(())
}
