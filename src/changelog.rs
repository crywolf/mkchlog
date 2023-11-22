//! Changelog creation logic

use crate::config::Command;
use crate::git::commit::Commit;
use crate::git::Git;
use crate::template::ChangelogTemplate;
use crate::template::Template;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::path::PathBuf;

const FORCE_CHECK_ALL_PROJECTS: &str = "force_check_all_projects";

/// Represents the generated changelog
pub struct Changelog<'a, T: ChangesList + Default> {
    template: &'a mut Template<T>,
    git: Git,
}

impl<'a, T> Changelog<'a, T>
where
    T: ChangesList + Default + Display,
{
    /// Creates a new [`Changelog`] object. Requires initialized [`Template`] and [`Git`] objects.
    pub fn new(template: &'a mut Template<T>, git: Git) -> Self {
        Self { template, git }
    }

    /// Generates the final changelog markdown string from the commit messages.
    pub fn generate(
        &mut self,
        project: Option<String>,
        command: Command,
    ) -> Result<String, Box<dyn Error>> {
        let mut project = project;
        let settings = &self.template.settings;
        let allowed_projects = &settings.projects_settings.projects.clone();
        let default_project_from_config = &settings.projects_settings.default_project.clone();
        let projects_since_commit = settings
            .projects_settings
            .since_commit
            .clone()
            .unwrap_or_default();

        let use_default_project = default_project_from_config.is_some();
        let mut default_project = &None;
        let mut set_default_project = false;

        // check if user provided project name matches the project name in YAML config file
        if let Some(project_name) = &project {
            if !&allowed_projects.contains_key(project_name) {
                return Err(
                    format!("Project '{}' not configured in config file", project_name).into(),
                );
            }
        }

        if command == Command::Check && !allowed_projects.is_empty() {
            // if we just are just checking commits in multi-project setting,
            // we need to check that all commits comply with the rules in template file
            project = Some(FORCE_CHECK_ALL_PROJECTS.to_string());
        }

        // get prepared general changelog structure from template YAML data
        let changelog_template = self.template.data();

        let commits = self.git.commits()?;

        // iterate through commits and fill in the changelog_template
        for commit in commits {
            // all commit until `since-commit` should belong to `default_project`
            if use_default_project {
                if set_default_project {
                    default_project = default_project_from_config;
                }
                if commit.commit_id == projects_since_commit {
                    set_default_project = true;
                }
            }

            let mut commit_changelog = CommitChangelog::new(commit);

            // insert changelog entries from commits to changelog_template
            commit_changelog.parse(
                changelog_template,
                allowed_projects,
                &project,
                default_project,
            )?;
        }

        // use prepared changelog_template and format the final changelog output
        let mut buff = String::new();

        if command == Command::Check {
            // just checking validity of commits, return empty String
            return Ok(buff);
        }

        // prepare and return changelog string
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
                for (_, subsec) in sec.subsections.iter() {
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
        allowed_projects: &HashMap<String, Vec<PathBuf>>,
        project: &Option<String>,
        default_project: &Option<String>,
    ) -> Result<(), Box<dyn Error>>
    where
        T: ChangesList + Default,
    {
        if self.changelog_lines().len() == 1 && self.changelog_lines()[0] == "skip" {
            return Ok(());
        }

        // if asking for a changelog for specific project, check if the commit belongs to it
        if let Some(project) = project.as_deref() {
            // if default project is set, then act as if it was specified in commit's changelog message
            // otherwise get it from the changelog message
            let changelog_project = if default_project.is_some() {
                default_project.as_ref().expect("default project is set")
            } else {
                self.get_key("project").ok_or(format!(
                    "Missing 'project' key in changelog message:\n>>> {}",
                    self.commit.raw_data
                ))?
            };

            if !allowed_projects.contains_key(changelog_project) {
                return Err(format!(
                    "Incorrect (not allowed in config file) project name '{}' in changelog message:\n>>> {}",
                    changelog_project, self.commit.raw_data
                )
                .into());
            }

            // return when commit belongs to different project than user asked for
            if changelog_project != project && project != FORCE_CHECK_ALL_PROJECTS {
                return Ok(());
            }
        }

        let mut title = self.get_key("title").unwrap_or("");
        let mut description = self.get_key("description").unwrap_or("");
        let title_is_enough = self.get_key("title-is-enough").unwrap_or("");
        let inherit = self.get_key("inherit").unwrap_or("");

        let section = self.get_key("section").ok_or(format!(
            "Missing 'section' key in changelog message:\n>>> {}",
            self.commit.raw_data
        ))?;
        let (section, sub_section) = section
            .split_once(':')
            .map(|(sec, subsec)| (sec.trim(), subsec.trim()))
            .unwrap_or((section, ""));

        if !changelog_template.contains_key(section) {
            if section.is_empty() {
                return Err(format!(
                    "Empty section in changelog message:\n>>> {}",
                    self.commit.raw_data
                )
                .into());
            }

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

                // remove hard wrapping (linefeeds) and indentation added by git in the description
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
                    unprivileged users you don't need to worry.";

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
                unprivileged users you don't need to worry.";

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
