//! Module for interaction with the `git` command

pub mod command;
pub mod commit;
pub mod stdin;

use self::commit::Commit;
use std::error::Error;

/// Trait that represents the `git log` command functionality
pub trait GitLogCommand {
    /// Returns the output of the `git log` command
    fn get_log(&self) -> Result<String, Box<dyn Error>>;
}

/// Git object for interaction with `git` command
pub struct Git {
    git_log_cmd: Box<dyn GitLogCommand>,
}

impl Git {
    /// Creates a new [`Git`] object that uses `git_log_cmd` to obtain the commits.
    pub fn new(git_log_cmd: Box<dyn GitLogCommand>) -> Self {
        Self { git_log_cmd }
    }

    /// Parses the output of the [`GitLogCommand`] and returns the collection of commits.
    pub fn commits(&self) -> Result<Vec<Commit>, Box<dyn Error>> {
        let git_log = self.git_log_cmd.get_log()?;

        // NB: `Regex::new(r"(?m)^commit [a-z|\d]{40}$")` was previously used to split the commits
        // however for some unknown reason it would cause `npm` to silently exit with success code when ran in WASM.
        // This workarounds the issue.
        let mut commits = Vec::new();
        if git_log.is_empty() {
            return Ok(commits);
        }

        let mut pos = 0;
        loop {
            let end = git_log[(pos + 1)..].find("\ncommit ");
            let copy_up_to = match end {
                Some(end) => pos + 1 + end,
                None => git_log.len(),
            };
            let commit = Commit::new(&git_log[pos..copy_up_to])?;
            commits.push(commit);
            if end.is_none() {
                break;
            } else {
                pos = copy_up_to;
            }
        }

        Ok(commits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct GitCmdMock;

    impl GitLogCommand for GitCmdMock {
        fn get_log(&self) -> Result<String, Box<dyn Error>> {
            let ouput = "\
commit a1a654e256cc96e1c4b5a81845b5e3f3f0aa9ed3
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:25:29 2023 +0200

    Fixed grammar mistakes.

    We found 42 grammar mistakes that are fixed in this commit.

    changelog: skip

commit 62db026b0ead7f0659df10c70e402c70ede5d7dd
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:24:22 2023 +0200

    Added ability to skip commits.

    This allows commits to be skipped by typing 'changelog: skip'
    at the end of the commit. This is mainly useful for typo
    fixes or other things irrelevant to the user of a project.

    changelog:
        section: features";

            Ok(ouput.to_string())
        }
    }

    #[test]
    fn git_commits() {
        let git = Git::new(Box::new(GitCmdMock));

        let res = git.commits().unwrap();
        assert_eq!(res.len(), 2);
    }
}
