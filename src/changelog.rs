//! Changelog creation logic

use crate::git::commit::Commit;
use crate::git::Git;
use crate::template::ChangelogTemplate;
use crate::template::Template;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::vec;

/// Represents the generated changelog
pub struct Changelog<T: ChangesList + Default> {
    template: Template<T>,
    git: Git,
}

impl<T> Changelog<T>
where
    T: ChangesList + Default + Display + Clone,
{
    /// Creates a new [`Changelog`] object. Requires initialized [`Template`] and [`Git`] objects.
    pub fn new(template: Template<T>, git: Git) -> Self {
        Self { template, git }
    }

    /// Generates the final changelog markdown string from the commit messages.
    pub fn generate(&self) -> Result<String, Box<dyn Error>> {
        // get prepared general changelog structure from template YAML data
        let mut changelog_template = self.template.data();

        let commits = self.git.commits()?;

        // iterate through commits and fill in the changelog_template
        for commit in commits {
            let mut commit_changelog = CommitChangelog::new(commit);

            // insert changelog entries from commits to changelog_template
            commit_changelog.parse(&mut changelog_template)?;
        }

        // use prepared changelog_template and format the final changelog output
        let mut buff = String::new();
        buff.push_str("============================================\n\n");

        for (_, sec) in changelog_template {
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

/// Changelog information provided in the commit message
struct CommitChangelog {
    commit: Commit,
    changelog_lines: Vec<String>,
}

impl CommitChangelog {
    /// Creates a new [`CommitChangelog`] from the given [`Commit`].
    fn new(commit: Commit) -> Self {
        let commit_changelog_lines = commit
            .changelog_message
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

        Self {
            commit,
            changelog_lines,
        }
    }

    /// Parses changelog section from the commit and fills it in the provided [`ChangelogTemplate`]
    pub fn parse<T>(
        &mut self,
        changelog_template: &mut ChangelogTemplate<T>,
    ) -> Result<(), Box<dyn Error>>
    where
        T: ChangesList + Default,
    {
        if self.changelog_lines().len() == 1 && self.changelog_lines()[0] == "skip" {
            return Ok(());
        }

        let section = self.get_key("section").ok_or(format!(
            "Missing 'section' key in changelog message:\n>>> {}",
            self.commit.raw_data
        ))?;
        let mut title = self.get_key("title").unwrap_or("");
        let mut description = self.get_key("description").unwrap_or("");
        let title_is_enough = self.get_key("title-is-enough").unwrap_or("");
        let inherit = self.get_key("inherit").unwrap_or("");

        let (section, sub_section) = section
            .split_once(':')
            .map(|(sec, subsec)| (sec, subsec.trim()))
            .unwrap_or((section, ""));

        if !changelog_template.contains_key(section) {
            return Err(format!(
                "Unknown section '{}' in changelog message:\n>>> {}",
                section, self.commit.raw_data
            )
            .into());
        }

        let commit_message_description: String;
        if inherit == "all"
            || inherit == "title"
            || (!title_is_enough.is_empty() && title.is_empty())
        {
            let re = Regex::new(r"\n\s*\n").expect("should never panic"); // title is separated by an empty line
            let mut commit_message_iter = re.splitn(&self.commit.message, 2);

            title = commit_message_iter
                .next()
                .map(|s| s.trim())
                .ok_or("Could not extract 'title' from commit message text")?;

            if description.is_empty() {
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
        }

        // we have title and description, we can insert them to changelog_template
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
            changelog_template
                .get_mut(section)
                .expect("section should be set correctly")
                .subsections
                .get_mut(sub_section)
                .expect("sub_section is not empty here")
                .changes
                .add(change_type, change);
        } else {
            changelog_template
                .get_mut(section)
                .expect("section should be set correctly")
                .changes
                .add(change_type, change);
        }

        Ok(())
    }

    /// Retuns the key from the changelog section in the commit
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

    // Return parsed lines from changelog section in the commit message
    fn changelog_lines(&self) -> &Vec<String> {
        self.changelog_lines.as_ref()
    }
}

/// Type of the changelog item
#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub enum ChangeType {
    /// Changelog item with title only
    TitleOnly,
    /// Changelog item with title and description
    Other,
}

/// List of changelog items in one section
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Changes {
    pub changes: HashMap<ChangeType, Vec<String>>,
}

impl Changes {
    /// Returns new list of changelog items in one section.
    fn new() -> Self {
        Self {
            changes: HashMap::from([(ChangeType::TitleOnly, vec![]), (ChangeType::Other, vec![])]),
        }
    }
}

pub trait ChangesList {
    /// Adds new item to the list of changes.
    fn add(&mut self, change_type: ChangeType, content: String);

    /// Returns `true` if the list of changes contains no elements.
    fn is_empty(&self) -> bool;
}

impl ChangesList for Changes {
    /// Adds new item to the list of changes.
    fn add(&mut self, change_type: ChangeType, content: String) {
        if let Some(v) = self.changes.get_mut(&change_type) {
            v.push(content);
        };
    }

    /// Returns `true` if the list of changes contains no elements.
    fn is_empty(&self) -> bool {
        self.changes
            .get(&ChangeType::TitleOnly)
            .expect("HashMap has all keys initialized")
            .is_empty()
            && self
                .changes
                .get(&ChangeType::Other)
                .expect("HashMap has all keys initialized")
                .is_empty()
    }
}

impl Default for Changes {
    fn default() -> Self {
        Self::new()
    }
}

use std::fmt;
impl fmt::Display for Changes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut only_title = String::new();
        if let Some(v) = self.changes.get(&ChangeType::TitleOnly) {
            only_title = v.concat();
        }

        let mut other = String::new();
        if let Some(v) = self.changes.get(&ChangeType::Other) {
            other = v.concat();
        }

        let mut result = String::new();
        result.push_str(&only_title);
        result.push_str(&other);

        write!(f, "{}", result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_changelog_with_multiple_lines_in_description() {
        let raw_data = "\
commit 1cc72956df91e2fd8c45e72983c4e1149f1ac3b3
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:27:59 2023 +0200

    Fixed TOCTOU race condition when opening file

    Previously we checked the file permissions before opening
    the file now we check the metadata using file descriptor
    after opening the file. (before reading)

    changelog:
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

        let commit = Commit::new(raw_data).unwrap();
        let commit_changelog = CommitChangelog::new(commit);
        let res = commit_changelog.changelog_lines();
        assert_eq!(res.len(), 3);
        assert_eq!(*res, exp);

        let section = commit_changelog.get_key("section").unwrap();
        assert_eq!(section, "security:vuln_fixes");

        let title = commit_changelog.get_key("title").unwrap();
        assert_eq!(title, "Fixed vulnerability related to opening files");

        // should not find non-present key ("descr" is not present, "description" is)
        assert!(
            commit_changelog.get_key("descr").is_none(),
            "should not find non-present key"
        );

        let descr = commit_changelog.get_key("description").unwrap();
        assert_eq!(descr, "The application was vulnerable to attacks if the attacker had access to the working directory. If you run this in such enviroment you should update ASAP. If your working directory is **not** accessible by unprivileged users you don't need to worry.");
    }

    #[test]
    fn commit_changelog_with_multiple_lines_in_title_and_description() {
        let raw_data = "\
commit 1cc72956df91e2fd8c45e72983c4e1149f1ac3b3
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:27:59 2023 +0200

    Fixed TOCTOU race condition when opening file

    changelog:
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

        let commit = Commit::new(raw_data).unwrap();
        let commit_changelog = CommitChangelog::new(commit);
        let res = commit_changelog.changelog_lines();
        assert_eq!(res.len(), 3);
        assert_eq!(*res, exp);
    }

    #[test]
    fn commit_changelog_without_multiple_lines() {
        let raw_data = "\
commit 1cc72956df91e2fd8c45e72983c4e1149f1ac3b3
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:27:59 2023 +0200

    Some feature

    changelog:
        inherit: all
        section: features";

        let exp = vec!["inherit: all", "section: features"];

        let commit = Commit::new(raw_data).unwrap();
        let commit_changelog = CommitChangelog::new(commit);
        let res = commit_changelog.changelog_lines();
        assert_eq!(res.len(), 2);
        assert_eq!(*res, exp);
    }
}
