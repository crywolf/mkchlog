//! Git commit

use regex::Regex;
use std::error::Error;

/// Git commit
#[derive(Debug)]
pub struct Commit {
    /// Git commit identifier (commit number)
    pub commit_id: String,
    /// Git commit header
    pub header: String,
    /// Git commit message
    pub message: String,
    /// Changelog message extracted from the commit message
    pub changelog_message: String,
    /// Raw data of the commit
    pub raw_data: String,
}

impl Commit {
    /// Parses raw data of the commit and returns a [`Commit`] object.
    pub fn new(raw_data: &str) -> Result<Self, Box<dyn Error>> {
        let data = &raw_data.trim().replace('\r', "")[..]; // remove extra \r in Windows

        let changelog_regex = Regex::new(r"(?m)^\s*changelog:").expect("should never panic");

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

        let changelog = commit_iter.next().map(str::trim).ok_or(format!(
            "Missing 'changelog:' key in commit:\n>>> {}",
            raw_data
        ))?;

        let commit_id = header
            .lines()
            .next()
            .ok_or(format!(
                "Could not parse commit id from commit:\n>>> {}",
                raw_data
            ))?
            .strip_prefix("commit ")
            .ok_or(format!(
                "Could not extract commit number from header:\n>>> {}",
                header
            ))?;

        let commit = Commit {
            commit_id: commit_id.to_owned(),
            header: header.to_owned(),
            message: commit_message.trim().to_owned(),
            changelog_message: changelog.to_owned(),
            raw_data: raw_data.to_owned(),
        };

        Ok(commit)
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
        title-is-enough: true";

        let exp_header = "commit 7c85bee4303d56bededdfacf8fbb7bdc68e2195b
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:26:35 2023 +0200";

        let exp_message = "Don't reallocate the buffer when we know its size

    This computes the size and allocates the buffer upfront.
    Avoiding allocations like this introduces 10% speedup.";

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

        let exp_message = "Don't reallocate the buffer when we know its size";

        let exp_changelog_message = "section: perf
        title: Improved processing speed by 10%
        title-is-enough: true";

        let res = Commit::new(raw_data).unwrap();
        assert_eq!(res.header, exp_header);
        assert_eq!(res.message, exp_message);
        assert_eq!(res.changelog_message, exp_changelog_message);
    }

    #[test]
    fn commit_new_missing_changelog_message() {
        let raw_data = "\
commit 7c85bee4303d56bededdfacf8fbb7bdc68e2195b
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:26:35 2023 +0200

    Don't reallocate the buffer when we know its size
";
        let res = Commit::new(raw_data);
        assert!(res.is_err());

        let exp_err = "\
Missing 'changelog:' key in commit:
>>> commit 7c85bee4303d56bededdfacf8fbb7bdc68e2195b
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:26:35 2023 +0200

    Don't reallocate the buffer when we know its size
";

        assert_eq!(res.unwrap_err().to_string(), exp_err);
    }
}
