//! Changelog creation logic

use crate::git::Git;
use crate::template::ChangeType;
use crate::template::Template;
use regex::Regex;
use std::error::Error;
use std::vec;

/// Represents the generated changelog
pub struct Changelog {
    template: Template,
    git: Git,
}

impl Changelog {
    /// Creates a new [`Changelog`] object. Requires initialized [`Template`] and [`Git`] objects.
    pub fn new(template: Template, git: Git) -> Self {
        Self { template, git }
    }

    /// Generates a changelog markdown string from the commit messages.
    pub fn produce(&self) -> Result<String, Box<dyn Error>> {
        // prepare changelog structure from template YAML data
        let mut changelog_map = self.template.data();

        // iterate through commits and fill in changelog_map
        let commits = self.git.commits()?;

        // insert changelog entries from commits to changelog_map
        for commit in commits {
            let commit_changelog_data = CommitChangelogData::new(&commit.changelog_message);

            let changelog_lines = commit_changelog_data.changelog_lines();
            if changelog_lines.len() == 1 && changelog_lines[0] == "skip" {
                continue;
            }

            let section = commit_changelog_data.get_key("section").ok_or(format!(
                "Missing 'section' key in changelog message:\n>>> {}",
                commit.raw_data
            ))?;
            let mut title = commit_changelog_data.get_key("title").unwrap_or("");
            let mut description = commit_changelog_data.get_key("description").unwrap_or("");

            let title_is_enough = commit_changelog_data
                .get_key("title-is-enough")
                .unwrap_or("");

            let inherit = commit_changelog_data.get_key("inherit").unwrap_or("");

            let (section, sub_section) = section
                .split_once(':')
                .map(|(sec, subsec)| (sec, subsec.trim()))
                .unwrap_or((section, ""));

            if !changelog_map.contains_key(section) {
                return Err(format!(
                    "Unknown section '{}' in changelog message:\n>>> {}",
                    section, commit.raw_data
                )
                .into());
            }

            let commit_message_description: String;
            if inherit == "all" || (!title_is_enough.is_empty() && title.is_empty()) {
                let re = Regex::new(r"\n\s*\n").expect("should never panic"); // title is separated by empty line
                let mut commit_message_iter = re.splitn(&commit.message, 2);

                title = commit_message_iter
                    .next()
                    .map(|s| s.trim())
                    .ok_or("Could not extract 'title' from commit message text")?;

                description = commit_message_iter
                    .next()
                    .map(|s| s.trim())
                    .unwrap_or_default();

                // remove hard wrapping (linefeeds) and identation added by git in the description
                let commit_message_description_lines: Vec<_> =
                    description.lines().map(|s| s.trim()).collect();
                commit_message_description = commit_message_description_lines.join(" ");
                description = &commit_message_description;
            }

            // we have title and description, we can insert them to changelog_map
            let title_prefix: &str;
            let mut change_type = ChangeType::Other;
            let mut change = String::new();

            if !title.is_empty() {
                if !title_is_enough.is_empty() || description.is_empty() {
                    change_type = ChangeType::TitleOnly;
                    title_prefix = "* ";
                } else if !sub_section.is_empty() {
                    title_prefix = "#### ";
                } else {
                    title_prefix = "### ";
                }
                change = title_prefix.to_owned();
                change.push_str(title);
                change.push_str("\n\n");
            }

            if !description.is_empty() && title_is_enough.is_empty() {
                change.push_str(description);
                change.push_str("\n\n");
            }

            if !sub_section.is_empty() {
                changelog_map
                    .get_mut(section)
                    .expect("section should be set correctly")
                    .subsections
                    .get_mut(sub_section)
                    .expect("sub_section is not empty here")
                    .changes
                    .add(change_type, change);
            } else {
                changelog_map
                    .get_mut(section)
                    .expect("section should be set correctly")
                    .changes
                    .add(change_type, change);
            }
        }

        // use prepared changelog_map and format changelog output
        let mut buff = String::new();
        buff.push_str("============================================\n\n");

        for (_, sec) in changelog_map {
            if !sec.changes.is_empty() || !sec.subsections.is_empty() {
                let mut print_section_header = !sec.changes.is_empty();
                for (_, subsec) in sec.subsections.iter() {
                    if !subsec.changes.is_empty() {
                        print_section_header = true;
                    }
                }

                if print_section_header {
                    buff.push_str("## ");
                    buff.push_str(&sec.title);
                    buff.push_str("\n\n");

                    if !sec.description.is_empty() {
                        buff.push_str(&sec.description);
                        buff.push_str("\n\n");
                    }
                }
            }

            if !sec.changes.is_empty() {
                buff.push_str(&sec.changes.to_string());
            }

            if !sec.subsections.is_empty() {
                for (_, subsec) in sec.subsections {
                    if !subsec.changes.is_empty() {
                        buff.push_str("### ");
                        buff.push_str(&subsec.title);
                        buff.push_str("\n\n");

                        if !subsec.description.is_empty() {
                            buff.push_str(&subsec.description);
                            buff.push_str("\n\n");
                        }
                    }

                    buff.push_str(&subsec.changes.to_string());
                }
            }
        }

        buff.push_str("============================================");

        Ok(buff)
    }
}

struct CommitChangelogData {
    changelog_lines: Vec<String>,
}

impl CommitChangelogData {
    fn new(commit_changelog: &str) -> Self {
        let commit_changelog_lines = commit_changelog
            .lines()
            .map(|s| s.trim())
            .collect::<Vec<_>>();

        let re = Regex::new(r"(?m)^[a-z-]+:").expect("should never panic"); // match keyword

        let mut changelog_lines: Vec<String> = vec![];

        for (i, &line) in commit_changelog_lines.iter().enumerate() {
            if i == 0 {
                changelog_lines.push(line.to_string());
                continue;
            }

            if !re.is_match(line) {
                // line does not start with keyword, append it to the previous one
                // ie. remove hard wrapping (linefeeds) inside changelog section in commit message
                let mut prev_line = changelog_lines.pop().unwrap_or_default();
                prev_line.push(' ');
                prev_line.push_str(line);
                changelog_lines.push(prev_line);
            } else {
                changelog_lines.push(line.to_string());
            }
        }

        Self { changelog_lines }
    }

    fn changelog_lines(&self) -> &Vec<String> {
        self.changelog_lines.as_ref()
    }

    fn get_key(&self, key: &str) -> Option<&str> {
        let key = key.to_owned() + ":";
        self.changelog_lines
            .iter()
            .map(|s| s.as_str())
            .find(|&line| line.starts_with(&key))
            .unwrap_or("")
            .split_once(':')
            .map(|(_, s)| s.trim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_changelog_with_multiple_lines_in_description() {
        let changelog_message = "\
section: security:vuln_fixes
title: Fixed vulnerability related to opening files
description: The application was vulnerable to attacks
             if the attacker had access to the working
             directory. If you run this in such
             enviroment you should update ASAP. If your
             working directory is **not** accessible by
             unprivileged users you don't need to worry.
";

        let exp = vec![
            "section: security:vuln_fixes",
            "title: Fixed vulnerability related to opening files",
            "description: The application was vulnerable to attacks if the attacker had access to the working directory. If you run this in such enviroment you should update ASAP. If your working directory is **not** accessible by unprivileged users you don't need to worry."];

        let commit_changelog_data = CommitChangelogData::new(changelog_message);
        let res = commit_changelog_data.changelog_lines();
        assert_eq!(res.len(), 3);
        assert_eq!(*res, exp);

        let section = commit_changelog_data.get_key("section").unwrap();
        assert_eq!(section, "security:vuln_fixes");

        let title = commit_changelog_data.get_key("title").unwrap();
        assert_eq!(title, "Fixed vulnerability related to opening files");

        // should not find non-present key ("descr" is not present, "description" is)
        assert!(
            commit_changelog_data.get_key("descr").is_none(),
            "should not find non-present key"
        );

        let descr = commit_changelog_data.get_key("description").unwrap();
        assert_eq!(descr, "The application was vulnerable to attacks if the attacker had access to the working directory. If you run this in such enviroment you should update ASAP. If your working directory is **not** accessible by unprivileged users you don't need to worry.");
    }

    #[test]
    fn commit_changelog_with_multiple_lines_in_title_and_description() {
        let changelog_message = "\
section: security:vuln_fixes
title: Fixed vulnerability related
to opening files
description: The application was vulnerable to attacks
         if the attacker had access to the working
         directory. If you run this in such
         enviroment you should update ASAP. If your
         working directory is **not** accessible by
         unprivileged users you don't need to worry.
";

        let exp = vec![
            "section: security:vuln_fixes",
            "title: Fixed vulnerability related to opening files",
            "description: The application was vulnerable to attacks if the attacker had access to the working directory. If you run this in such enviroment you should update ASAP. If your working directory is **not** accessible by unprivileged users you don't need to worry."];

        let commit_changelog_data = CommitChangelogData::new(changelog_message);
        let res = commit_changelog_data.changelog_lines();
        assert_eq!(res.len(), 3);
        assert_eq!(*res, exp);
    }

    #[test]
    fn commit_changelog_without_multiple_lines() {
        let changelog_message = "\
  inherit: all
  section: features";

        let exp = vec!["inherit: all", "section: features"];

        let commit_changelog_data = CommitChangelogData::new(changelog_message);
        let res = commit_changelog_data.changelog_lines();
        assert_eq!(res.len(), 2);
        assert_eq!(*res, exp);
    }
}
