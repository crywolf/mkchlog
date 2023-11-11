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

    let mut template = Template::<changelog::Changes>::new(f)?;

    // set value from program arguments or yaml file
    let commit_id = match (
        config.commit_id,
        template.settings.skip_commits_up_to.as_ref(),
    ) {
        (Some(commit_id), _) => Some(commit_id),
        (None, Some(commit_id)) => Some(commit_id.to_owned()),
        (None, None) => None,
    };

    // set value from program arguments or yaml file
    let git_path = match (config.git_path, template.settings.git_path.as_ref()) {
        (Some(git_path), _) => git_path,
        (_, Some(git_path)) => git_path.to_owned(),
        (None, None) => std::path::PathBuf::from("./"),
    };

    match (
        &config.project,
        &template.settings.projects_settings.projects,
    ) {
        (None, projects) => {
            if !projects.is_empty() && config.command == Command::Generate {
                // 'project' arg can be empty when we are just checking the commits, not generating a changelog
                return Err(
                    "You need to specify project name. Use command 'help' for more information."
                        .into(),
                );
            }
        }
        (Some(proj), projects) => {
            if projects.is_empty() {
                return Err(format!(
                    "Omit project option '{}', repository is not configured as multi-project.",
                    proj
                )
                .into());
            }
        }
    }

    let git = if config.read_from_stdin {
        use git::stdin::Stdin;
        let git_cmd = Box::new(Stdin::new());
        Git::new(git_cmd)
    } else {
        let git_cmd = Box::new(GitLogCmd::new(git_path, commit_id));
        Git::new(git_cmd)
    };

    let mut changelog = Changelog::new(&mut template, git);
    let output = changelog.generate(config.project, config.command)?;
    println!("{}", output);

    Ok(())
}
