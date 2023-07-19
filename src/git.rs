use commit::Commit;
use regex::Regex;
use std::error::Error;

pub struct Git {
    path: String,
}

impl Git {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    fn get_log(&self) -> Result<String, Box<dyn Error>> {
        let mut git_command = std::process::Command::new("git");
        git_command.arg("-C").arg(&self.path).arg("log");

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

    pub fn commits(&self) -> Result<Vec<Commit>, Box<dyn Error>> {
        let git_log = self.get_log()?;

        let commit_regex = Regex::new(r"(?m)^commit [a-z|\d]{40}$").unwrap();

        let mut matches = commit_regex.find_iter(&git_log); // matches all lines with commit numbers

        let commits: Result<Vec<Commit>, _> = commit_regex
            .split(&git_log) // split by lines with commit numbers-
            .skip(1) // first element is empty
            .map(|s| {
                let m = matches
                    .next() // get lone with commit number and prepend it to the raw commit data
                    .ok_or("Could not parse git log output (commit number)")?;

                let mut r = m.as_str().to_owned();
                r.push_str(s);

                Commit::new(&r)
            })
            .collect();

        commits
    }
}

mod commit {
    use regex::Regex;
    use std::error::Error;

    pub struct Commit {
        pub header: String,
        pub message: String,
        pub changelog_message: String,
        pub raw_data: String,
    }

    impl Commit {
        pub fn new(raw_data: &str) -> Result<Self, Box<dyn Error>> {
            let data = &raw_data.replace('\r', "")[..]; // remove extra \r in Windows

            let changelog_regex = Regex::new(r"(?m)^\s*changelog:").unwrap();

            let mut commit_iter = changelog_regex.split(data);

            let (header, commit_message) = commit_iter
                .next()
                .ok_or(format!(
                    "Could not parse commit message in commit:\n>>> {}",
                    raw_data
                ))?
                .split_once("\n\n")
                .ok_or(format!(
                    "Could not extract commit message text in commit:\n>>> {}",
                    raw_data
                ))?;

            let changelog: String = commit_iter.map(|s| s.trim()).collect();
            if changelog.is_empty() {
                return Err(
                    format!("Missing 'changelog:' key in commit:\n>>> {}", raw_data).into(),
                );
            }

            let commit = Commit {
                header: header.to_owned(),
                message: commit_message.to_owned(),
                changelog_message: changelog,
                raw_data: raw_data.to_owned(),
            };

            Ok(commit)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_new() {
        let raw_data = "commit 7c85bee4303d56bededdfacf8fbb7bdc68e2195b
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:26:35 2023 +0200

    Don't reallocate the buffer when we know its size

    This computes the size and allocates the buffer upfront.
    Avoiding allocations like this introduces 10% speedup.

    changelog:
        section: perf
        title: Improved processing speed by 10%
        title-is-enough: true

";

        let exp_header = "commit 7c85bee4303d56bededdfacf8fbb7bdc68e2195b
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:26:35 2023 +0200";

        let exp_message = "    Don't reallocate the buffer when we know its size

    This computes the size and allocates the buffer upfront.
    Avoiding allocations like this introduces 10% speedup.
";

        let exp_changelog_message = "section: perf
        title: Improved processing speed by 10%
        title-is-enough: true";

        let res = Commit::new(raw_data).unwrap();
        assert_eq!(res.header, exp_header);
        assert_eq!(res.message, exp_message);
        assert_eq!(res.changelog_message, exp_changelog_message);
    }

    #[test]
    fn commit_new_with_windows_extra_carrige_return() {
        // commit with \r\n as a line separator
        let raw_data = "commit 7c85bee4303d56bededdfacf8fbb7bdc68e2195b\r\nAuthor: Cry Wolf <cry.wolf@centrum.cz>\r\nDate:   Tue Jun 13 16:26:35 2023 +0200\r\n\r\n    Don't reallocate the buffer when we know its size\r\n    changelog:\r\n        section: perf\r\n        title: Improved processing speed by 10%\r\n        title-is-enough: true";

        let exp_header = "commit 7c85bee4303d56bededdfacf8fbb7bdc68e2195b
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:26:35 2023 +0200";

        let exp_message = "    Don't reallocate the buffer when we know its size
";

        let exp_changelog_message = "section: perf
        title: Improved processing speed by 10%
        title-is-enough: true";

        let res = Commit::new(raw_data).unwrap();
        assert_eq!(res.header, exp_header);
        assert_eq!(res.message, exp_message);
        assert_eq!(res.changelog_message, exp_changelog_message);
    }
}
