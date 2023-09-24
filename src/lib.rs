//! Changelog generator tool suitable for user-facing changelogs and based on experiences of existing projects.
//!
//! Refer to `README.md` for more information

pub mod changelog;
pub mod config;
pub mod git;
pub mod template;

use crate::changelog::Changelog;
use crate::config::Command;
use crate::git::command::GitLogCmd;
use crate::git::Git;
use crate::template::Template;
use std::fs::File;

/// Entrypoint of the application
pub fn run(config: config::Config) -> Result<(), Box<dyn std::error::Error>> {
    let f = match File::open(&config.file_path) {
        Ok(f) => f,
        Err(err) => {
            return Err(format!(
                "Error reading config YAML file '{}': {}",
                config.file_path.display(),
                err
            )
            .into())
        }
    };

    let template = Template::new(f)?;

    let git_cmd = Box::new(GitLogCmd::new(config.git_path, config.commit_id));
    let git = Git::new(git_cmd);

    let changelog = Changelog::new(template, git);

    let output = changelog.produce()?;

    if let Command::Generate = config.command {
        println!("{}", output);
    }

    Ok(())
}
