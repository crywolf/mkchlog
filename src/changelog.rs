use crate::git::Git;
use crate::template::Template;
use indexmap::IndexMap;
use regex::Regex;
use std::error::Error;
use std::vec;

pub struct Changelog {
    template: Template,
    git: Git,
}

struct Section {
    title: String,
    description: String,
    subsections: IndexMap<String, Section>,
    changes: String,
}

impl Changelog {
    pub fn new(template: Template, git: Git) -> Changelog {
        Self { template, git }
    }

    pub fn produce(&self) -> Result<String, Box<dyn Error>> {
        // parsing template YAML data
        let tmpl_sections_key = match self.template.data().get("sections") {
            Some(v) => v,
            None => return Err("Missing 'sections' key in config file".into()),
        };

        let tmpl_sections = tmpl_sections_key
            .as_mapping()
            .ok_or("Malformed 'sections' key in config file")?;

        let mut changelog_map = IndexMap::<String, Section>::new();

        // prepare changelog structure from template YAML data
        for (sec, val) in tmpl_sections {
            let sec = sec.as_str().ok_or("Invalid section")?.to_owned();
            let val = val
                .as_mapping()
                .ok_or(format!("Invalid value in section '{}' in config file", sec))?;

            let title = val
                .get("title")
                .ok_or(format!(
                    "Missing 'title' in section '{}' in config file",
                    sec
                ))?
                .as_str()
                .ok_or(format!(
                    "Invalid 'title' in section '{}' in config file",
                    sec
                ))?;

            let mut description = "".to_owned();
            if let Some(descr) = val.get("description") {
                description = descr.as_str().unwrap_or("").to_string();
            }

            let mut section = Section {
                title: title.to_string(),
                description: description.to_string(),
                subsections: IndexMap::new(),
                changes: String::new(),
            };

            if let Some(subsections) = val.get("subsections") {
                let mut sub_section_map = IndexMap::<String, String>::new();
                sub_section_map.insert("title".to_string(), title.to_string());

                let subsections_map: Result<IndexMap<String, Section>, String> = subsections
                    .as_mapping()
                    .ok_or(format!(
                        "Invalid subsections format in section {} in config file",
                        sec
                    ))?
                    .iter()
                    .map(|(key, val)| {
                        let subsection_name = key.as_str().ok_or(format!(
                            "Invalid subsection in section '{}' in config file",
                            sec
                        ))?;

                        let title = val
                            .get("title")
                            .ok_or(format!(
                                "Missing 'title' in section '{}' in config file",
                                subsection_name
                            ))?
                            .as_str()
                            .ok_or(format!(
                                "Invalid 'title' in section '{}' in config file",
                                subsection_name
                            ))?;

                        let mut description = "";
                        if let Some(descr) = val.get("description") {
                            description = descr.as_str().unwrap_or("");
                        }

                        Ok((
                            subsection_name.to_string(),
                            Section {
                                title: title.to_string(),
                                description: description.to_string(),
                                subsections: IndexMap::new(),
                                changes: String::new(),
                            },
                        ))
                    })
                    .collect();
                section.subsections = subsections_map?;
            }

            changelog_map.insert(sec.to_string(), section);
        }

        // iterate through commits and fill in changelog_map
        let commits = self.git.commits()?;

        // insert changelog entries from commits
        for commit in commits {
            let mut changes = String::new();

            let changelog_lines = changelog_lines(&commit.changelog_message);

            if changelog_lines.len() == 1 && changelog_lines[0] == "skip" {
                continue;
            }

            let section = self.get_key(&changelog_lines, "section:").ok_or(format!(
                "Missing 'section' key in changelog message:\n>>> {}",
                commit.raw_data
            ))?;

            let mut title = self.get_key(&changelog_lines, "title:").unwrap_or("");

            let mut description = self.get_key(&changelog_lines, "description:").unwrap_or("");

            let title_is_enough = self
                .get_key(&changelog_lines, "title-is-enough:")
                .unwrap_or("");

            let inherit = self.get_key(&changelog_lines, "inherit:").unwrap_or("");

            let (section, sub_section) = section
                .split_once(':')
                .map(|(sec, subsec)| (sec, subsec.trim()))
                .unwrap_or((section, ""));

            ///////////////////

            if !changelog_map.contains_key(section) {
                return Err(format!(
                    "Unknown section '{}' in changelog messsage:\n>>> {}",
                    section, commit.raw_data
                )
                .into());
            }

            if inherit == "all" {
                let re = Regex::new(r"\n\s*\n").unwrap();
                let mut commit_message_iter = re.split(&commit.message);

                title = commit_message_iter
                    .next()
                    .map(|s| s.trim())
                    .ok_or("Could not extract 'title' from commit message text")?;

                // TODO - remove hard wrapping (linefeeds) in the description?
                description = commit_message_iter
                    .next()
                    .map(|s| s.trim())
                    .ok_or("Could not extract 'description' from commit message text")?;
            }

            if !title.is_empty() {
                if !title_is_enough.is_empty() {
                    changes.push_str("* ");
                } else if !sub_section.is_empty() {
                    changes.push_str("#### ");
                } else {
                    changes.push_str("### ");
                }
                changes.push_str(title);
            }

            if !description.is_empty() {
                changes.push_str("\n\n");
                changes.push_str(description);
            }

            if !sub_section.is_empty() {
                changelog_map
                    .get_mut(section)
                    .unwrap() // TODO
                    .subsections
                    .get_mut(sub_section)
                    .unwrap() // TODO
                    .changes = changes;
            } else {
                changelog_map.get_mut(section).unwrap().changes = changes;
            }
        }

        ////////////////////////////
        let mut buff = String::new();
        buff.push_str("============================================");

        for (_, sec) in changelog_map.into_iter() {
            buff.push_str("\n## ");
            buff.push_str(&sec.title);

            if !sec.description.is_empty() {
                buff.push_str("\n\n");
                buff.push_str(&sec.description);
                buff.push('\n');
            }

            if !sec.changes.is_empty() {
                buff.push_str("\n\n");
                buff.push_str(&sec.changes);
                buff.push('\n');
            }

            if !sec.subsections.is_empty() {
                for (_, subsec) in sec.subsections {
                    buff.push_str("\n### ");
                    buff.push_str(&subsec.title);

                    if !subsec.description.is_empty() {
                        buff.push_str("\n\n");
                        buff.push_str(&subsec.description);
                        buff.push('\n');
                    }

                    if !subsec.changes.is_empty() {
                        buff.push_str("\n\n");
                        buff.push_str(&subsec.changes);
                        buff.push('\n');
                    }
                }
            }
        }

        buff.push_str("============================================");

        Ok(buff)
    }

    // TODO - refactor as a method of Commit struct
    fn get_key<'a>(&'a self, from: &'a [String], key: &str) -> Option<&str> {
        from.iter()
            .map(|s| s.as_str())
            .find(|&line| line.starts_with(key))
            .unwrap_or("")
            .split_once(':')
            .map(|(_, s)| s.trim())
    }
}

// TODO - refactor as a method of Commit struct
fn changelog_lines(changelog: &str) -> Vec<String> {
    let orig_changelog_lines = changelog.lines().map(|s| s.trim()).collect::<Vec<_>>();

    let re = Regex::new(r"(?m)^[a-z-]+:").unwrap(); // match keyword

    let mut changelog_lines: Vec<String> = vec![];

    for (i, &line) in orig_changelog_lines.iter().enumerate() {
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

    changelog_lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changelog_lines_with_multiple_lines_in_description() {
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

        let res = changelog_lines(changelog_message);
        assert_eq!(res.len(), 3);
        assert_eq!(res, exp);
    }

    #[test]
    fn changelog_lines_with_multiple_lines_in_title_and_description() {
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

        let res = changelog_lines(changelog_message);
        assert_eq!(res.len(), 3);
        assert_eq!(res, exp);
    }

    #[test]
    fn changelog_lines_without_multiple_lines() {
        let changelog_message = "\
  inherit: all
  section: features";

        let exp = vec!["inherit: all", "section: features"];

        let res = changelog_lines(changelog_message);
        assert_eq!(res.len(), 2);
        assert_eq!(res, exp);
    }
}
