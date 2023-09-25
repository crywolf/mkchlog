//! `git log` command implementation

use std::error::Error;
use std::path::PathBuf;

/// Represents the `git log` command
pub struct GitLogCmd {
    path: PathBuf,
    commit_id: Option<String>,
}

impl GitLogCmd {
    /// Creates a new [`GitLogCmd`]. Accepts the path to the `git` repository and optional commit number.
    pub fn new(path: PathBuf, commit_id: Option<String>) -> Self {
        Self { path, commit_id }
    }
}

impl super::GitLogCommand for GitLogCmd {
    fn get_log(&self) -> Result<String, Box<dyn Error>> {
        let mut git_command = std::process::Command::new("git");
        git_command.arg("-C").arg(&self.path).arg("log");

        if self.commit_id.is_some() {
            // add argument: git log 7c85bee4303d56bededdfacf8fbb7bdc68e2195b..HEAD
            git_command.arg(format!(
                "{}..HEAD",
                self.commit_id.as_ref().expect("commit_id is not empty")
            ));
        }

        let git_cmd_output = git_command.output().map_err(|err| {
            format!(
                "Failed to execute '{}' command: {}",
                git_command.get_program().to_str().unwrap_or("git"),
                err
            )
        })?;

        if !git_cmd_output.status.success() {
            let args: Vec<_> = git_command
                .get_args()
                .map(|a| a.to_str().unwrap_or("git log"))
                .collect();

            return Err(format!(
                "Failed to execute 'git {}' command:\n{}",
                args.join(" "),
                String::from_utf8_lossy(&git_cmd_output.stderr).into_owned()
            )
            .into());
        }

        let git_log = String::from_utf8_lossy(&git_cmd_output.stdout);

        Ok(git_log.into_owned())
    }
}
