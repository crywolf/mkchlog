pub mod command;
mod commit;

use self::commit::Commit;
use regex::Regex;
use std::error::Error;

pub trait GitLogCommand {
    fn get_log(&self) -> Result<String, Box<dyn Error>>;
}

pub struct Git {
    git_log_cmd: Box<dyn GitLogCommand>,
}

impl Git {
    pub fn new(git_log_cmd: Box<dyn GitLogCommand>) -> Self {
        Self { git_log_cmd }
    }

    pub fn commits(&self) -> Result<Vec<Commit>, Box<dyn Error>> {
        let git_log = self.git_log_cmd.get_log()?;

        let commit_regex = Regex::new(r"(?m)^commit [a-z|\d]{40}$").unwrap();

        let mut matches = commit_regex.find_iter(&git_log); // matches all lines with commit numbers

        let commits: Result<Vec<Commit>, _> = commit_regex
            .split(&git_log) // split by lines with commit numbers-
            .skip(1) // first element is empty
            .map(|s| {
                let m = matches
                    .next() // get line with commit number and prepend it to the raw commit data
                    .ok_or("Could not parse git log output (commit number)")?;

                let mut r = m.as_str().to_owned();
                r.push_str(s);

                Commit::new(&r)
            })
            .collect();

        commits
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
        inherit: all
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
