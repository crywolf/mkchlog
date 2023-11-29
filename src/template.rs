//! Template represents parsed YAML config file
use indexmap::IndexMap;
use serde::Deserialize;
use serde_yaml::Value;
use std::error::Error;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Template represents parsed YAML config file
#[derive(Debug)]
pub struct Template<T: Default> {
    changelog_template: ChangelogTemplate<T>,
    pub settings: Settings,
}

/// Settings represent options that were set in YAML config file
#[derive(Debug)]
pub struct Settings {
    pub skip_commits_up_to: Option<String>,
    pub git_path: Option<PathBuf>,
    pub projects_settings: ProjectsSettings,
}

/// Multi-project repository settings from YAML config file
#[derive(Debug)]
pub struct ProjectsSettings {
    pub projects: IndexMap<String, Project>,
    pub since_commit: Option<String>,
    pub default_project: Option<String>,
}

/// Data structure prefiled from YAML config with section names as keys
/// and empty [`Section`]s to be filled with changelog messages from commits
pub type ChangelogTemplate<T> = IndexMap<String, Section<T>>;
type Yaml = serde_yaml::Value;

impl<T: Default> Template<T> {
    /// Parses the config (template) YAML file and returns the initialized template object.
    pub fn new(mut file: impl Read) -> Result<Self, Box<dyn Error>> {
        let mut config_yml = String::new();
        file.read_to_string(&mut config_yml)?;

        Self::from_str(&config_yml)
    }

    /// Validates template data extracted from the configuration (template) file
    /// and prepares data structure for storing changelog data.
    fn parse_config(&mut self, yaml: Yaml) -> Result<(), Box<dyn Error>> {
        // parsing template YAML data
        let tmpl_sections_key = match yaml.get("sections") {
            Some(v) => v,
            None => return Err("Missing 'sections' key in config file".into()),
        };

        let tmpl_sections = tmpl_sections_key
            .as_mapping()
            .ok_or("Malformed 'sections' key in config file")?;

        for (sec, val) in tmpl_sections {
            let sec = sec.as_str().ok_or("Invalid section")?.to_owned();
            let val = val
                .as_mapping()
                .ok_or(format!("Invalid value in section '{}' in config file", sec))?;

            let title = val
                .get(&Value::from("title"))
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
            if let Some(descr) = val.get(&Value::from("description")) {
                description = descr.as_str().unwrap_or("").to_string();
            }

            let mut section = Section {
                title: title.to_string(),
                description: description.to_string(),
                subsections: IndexMap::new(),
                changes: T::default(),
            };

            if let Some(subsections) = val.get(&Value::from("subsections")) {
                let mut sub_section_map = IndexMap::<String, String>::new();
                sub_section_map.insert("title".to_string(), title.to_string());

                let subsections_map: Result<IndexMap<String, Section<T>>, String> = subsections
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
                                changes: T::default(),
                            },
                        ))
                    })
                    .collect();
                section.subsections = subsections_map?;
            }

            self.changelog_template.insert(sec.to_string(), section);
        }

        Ok(())
    }

    /// Generates commit template for git from the config file
    pub fn generate_commit_template(
        &self,
        mut changed_files: impl Read,
    ) -> Result<String, Box<dyn Error>> {
        let mut out = String::new();
        let mut projects: Vec<&str> = vec![];
        if !self.settings.projects_settings.projects.is_empty() {
            let mut buf = String::new();
            changed_files.read_to_string(&mut buf)?;

            let allowed_projects = &self.settings.projects_settings.projects;

            // determine project name from directory/directories of committed files
            for line in buf.lines() {
                let file_path = PathBuf::from(line);
                let mut file_found = false;

                for (name, project) in allowed_projects {
                    let allowed_dirs = &project.dirs;
                    for dir in allowed_dirs {
                        if file_found {
                            break;
                        }

                        if dir == &PathBuf::from(".") && file_path.parent() == Some(Path::new("")) {
                            // file is in main directory
                            if !projects.contains(&name.as_str()) {
                                projects.push(name);
                            }
                            file_found = true;
                            break;
                        }
                        if file_path.starts_with(dir) {
                            // file is in subdirectory
                            if !projects.contains(&name.as_str()) {
                                projects.push(name);
                            }
                            file_found = true;
                            break;
                        }
                    }
                }
                if !file_found {
                    return Err(format!(
                        "Could not determine project for file: '{}'. Is the directory correctly set in the config file?",
                        file_path.display()
                    )
                    .into());
                }
            }
        }

        out.push_str("\n\n");
        out.push_str("changelog:\n");
        if !projects.is_empty() {
            if projects.len() == 1 {
                out.push_str("  project: ");
                out.push_str(projects[0]);
                out.push('\n');
                out.push_str("  section:\n");
                out.push_str("  inherit: all\n");
            } else {
                for project in projects {
                    out.push_str(" - project:\n");
                    out.push_str("    name: ");
                    out.push_str(project);
                    out.push('\n');
                    out.push_str("    section:\n");
                    out.push_str("    inherit: all\n");
                }
            }
        } else {
            out.push_str("  section:\n");
            out.push_str("  inherit: all\n");
        }

        out.push_str("#\n");
        out.push_str("# Valid changelog sections:\n#");

        // find longest section+subsection name for indentation
        let mut longest_section_name_len = 0;
        for (keyword, sec) in &self.changelog_template {
            let mut new_len = keyword.len();
            for (keyword, _) in sec.subsections.iter() {
                new_len += keyword.len() + 1; // plus 1 for subsection separator
            }
            if new_len > longest_section_name_len {
                longest_section_name_len = new_len;
            }
        }

        let spaces = 2;
        for (keyword, sec) in &self.changelog_template {
            let keyword_len = keyword.len();
            out.push('\n');
            out.push_str("# * ");
            out.push_str(keyword);

            if sec.subsections.is_empty() {
                let indentation = longest_section_name_len - keyword_len + spaces;
                for _i in 0..indentation {
                    out.push(' ');
                }
                out.push_str(&sec.title);
            } else {
                for (keyword, subsec) in sec.subsections.iter() {
                    let sub_keyword_len = keyword.len();
                    let indentation =
                        longest_section_name_len - keyword_len - sub_keyword_len - 1 + spaces;

                    out.push('.');
                    out.push_str(keyword);
                    for _i in 0..indentation {
                        out.push(' ');
                    }

                    out.push_str(&subsec.title);
                }
            }
        }

        Ok(out)
    }

    /// Returns mutable reference to the data structure with initialized sections for storing changelog data.
    pub fn data(&mut self) -> &mut ChangelogTemplate<T> {
        &mut self.changelog_template
    }
}

impl<T: Default> std::str::FromStr for Template<T> {
    type Err = Box<dyn Error>;

    fn from_str(config_yml: &str) -> Result<Self, Self::Err> {
        let config: Yaml = match serde_yaml::from_str(config_yml) {
            Ok(config) => config,
            Err(err) => return Err(format!("Error parsing config YAML file: {}", err).into()),
        };

        let skip_commits_up_to = config
            .get("skip-commits-up-to")
            .map(|v| {
                v.as_str()
                    .map(ToOwned::to_owned)
                    .ok_or("'skip-commits-up-to' key in config file must be a string")
            })
            .transpose()?;

        let git_path = config
            .get("git-path")
            .map(|v| {
                v.as_str()
                    .map(std::path::PathBuf::from)
                    .ok_or("'git-path' key in config file must be a string")
            })
            .transpose()?;

        let projects_sec = config
            .get("projects")
            .map(|v| {
                v.as_mapping()
                    .map(ToOwned::to_owned)
                    .ok_or("Malformed 'projects' key in config file")
            })
            .transpose()?;

        let mut projects: IndexMap<String, Project> = IndexMap::new();
        let mut since_commit = None;
        let mut default_project = None;

        if let Some(projects_sec) = projects_sec {
            let projects_sec_list = projects_sec
                .get(&Value::from("list"))
                .map(|v| {
                    v.as_sequence().map(ToOwned::to_owned).ok_or(
                        "'list' in 'projects' in config file must be an array (list of projects)",
                    )
                })
                .transpose()?
                .ok_or("Missing 'list' key in config file")?;

            let projects_list = projects_sec_list
                .iter()
                .map(|v| serde_yaml::from_value::<ProjectWrapper>(v.clone()))
                .collect::<Result<Vec<_>, _>>();

            if projects_list.is_err() {
                return Err(format!(
                    "Malformed list of projects in config file: {}",
                    projects_list.expect_err("we have error message")
                )
                .into());
            }

            let projects_list: Vec<Project> =
                projects_list?.into_iter().map(|pw| pw.project).collect();

            for project in projects_list {
                projects.insert(project.name.clone(), project);
            }

            since_commit = projects_sec
                .get(&Value::from("since-commit"))
                .map(|v| {
                    v.as_str()
                        .map(ToOwned::to_owned)
                        .ok_or("'since-commits' key in config file must be a string")
                })
                .transpose()?;

            default_project = projects_sec
                .get(&Value::from("default"))
                .map(|v| {
                    v.as_str()
                        .map(ToOwned::to_owned)
                        .ok_or("'default' key in config file must be a string")
                })
                .transpose()?;

            if since_commit.is_some() && default_project.is_none() {
                return Err("Default project name is not set config file".into());
            }

            if default_project.is_some()
                && !projects.contains_key(default_project.as_ref().expect("default name is set"))
            {
                return Err("Default project name is not contained in project names list".into());
            }
        }

        let mut template = Self {
            changelog_template: ChangelogTemplate::new(),
            settings: Settings {
                skip_commits_up_to,
                git_path,
                projects_settings: ProjectsSettings {
                    projects,
                    since_commit,
                    default_project,
                },
            },
        };

        template.parse_config(config)?;

        Ok(template)
    }
}

#[derive(Debug, Deserialize)]
pub struct ProjectWrapper {
    project: Project,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Project {
    name: String,
    dirs: Vec<PathBuf>,
}

/// Data structure to store changelog section data
#[derive(Debug, Clone, PartialEq)]
pub struct Section<T: Default> {
    pub title: String,
    pub description: String,
    pub subsections: IndexMap<String, Section<T>>,
    pub changes: T,
}

#[cfg(test)]
mod tests {
    use super::Template;
    use crate::changelog::Changes;
    use std::io::Cursor;

    pub struct FileReaderMock {
        content: Cursor<String>,
    }

    impl FileReaderMock {
        pub fn new(content: &str) -> Self {
            Self {
                content: Cursor::new(content.to_owned()),
            }
        }
    }

    impl std::io::Read for FileReaderMock {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.content.read(buf)
        }
    }

    const YAML_TEMPLATE: &str = r#"
    skip-commits-up-to: bc58e6bf2cf640d46aa832e297d0f215f76dfce0

    projects:
      list:
        - project:
            name: main
            dirs: [".", .github, .githooks]
        - project:
            name: mkchlog
            dirs: [mkchlog]
        - project:
            name: mkchlog-action
            dirs: [mkchlog-action]

      since-commit: 276aa9e4b013de1646ea57cfcbf74e5966524f68 # projects are mandatory since COMMIT_NUMBER
      default: mkchlog # commits up to COMMIT_NUMBER are considered belonging to the project NAME

    sections:
        # section identifier selected by project maintainer
        security:
            # The header presented to the user
            title: Security
            # desctiption is optional and will appear above changes
            description: This section contains very important security-related changes.
            subsections:
                vuln_fixes:
                    title: Fixed vulnerabilities
        features:
            # some comment
            title: New features
        bug_fixes:
            title: Fixed bugs
        breaking:
            title: Breaking changes
        perf:
            title: Performance improvements
        dev:
            title: Development
            description: Internal development changes
    "#;

    #[test]
    fn template_valid_yaml() {
        use super::Section;
        use indexmap::IndexMap;

        let f = FileReaderMock::new(YAML_TEMPLATE);

        let res = Template::new(f);
        assert!(res.is_ok());

        let mut template = res.unwrap();

        // check for correctly parsed settings
        let settings = &template.settings;
        assert_eq!(
            settings.skip_commits_up_to.as_ref().unwrap(),
            "bc58e6bf2cf640d46aa832e297d0f215f76dfce0"
        );

        assert_eq!(settings.projects_settings.projects.len(), 3);
        for key in settings.projects_settings.projects.keys() {
            let proj = settings.projects_settings.projects.get(key).unwrap();
            assert_eq!(*key, proj.name);
            if key == "main" {
                assert_eq!(proj.dirs.len(), 3);
            } else {
                assert_eq!(proj.dirs.len(), 1);
            }
        }

        assert_eq!(
            settings.projects_settings.since_commit.as_ref().unwrap(),
            "276aa9e4b013de1646ea57cfcbf74e5966524f68"
        );
        assert_eq!(
            settings.projects_settings.default_project.as_ref().unwrap(),
            "mkchlog"
        );

        // check if parsed template has correct format
        let template_data = template.data();

        let exp_keys = template_data.keys().collect::<Vec<_>>();
        assert_eq!(exp_keys.len(), 6);
        assert_eq!(
            exp_keys,
            vec![
                "security",
                "features",
                "bug_fixes",
                "breaking",
                "perf",
                "dev",
            ]
        );

        let exp_sections = template_data.values().cloned().collect::<Vec<_>>();
        assert_eq!(exp_sections.len(), 6);

        // 'security' section with subsection
        let mut subsecs = IndexMap::new();
        subsecs.insert(
            "vuln_fixes".to_owned(),
            Section {
                title: "Fixed vulnerabilities".to_owned(),
                description: "".to_owned(),
                subsections: IndexMap::new(),
                changes: Changes::default(),
            },
        );
        assert_eq!(
            exp_sections[0],
            Section {
                title: "Security".to_owned(),
                description: "This section contains very important security-related changes."
                    .to_owned(),
                subsections: subsecs,
                changes: Changes::default(),
            }
        );

        // 'features' section
        assert_eq!(
            exp_sections[1],
            Section {
                title: "New features".to_owned(),
                description: "".to_owned(),
                subsections: IndexMap::new(),
                changes: Changes::default(),
            }
        );

        // 'dev' section
        assert_eq!(
            exp_sections[5],
            Section {
                title: "Development".to_owned(),
                description: "Internal development changes".to_owned(),
                subsections: IndexMap::new(),
                changes: Changes::default(),
            }
        );
    }

    #[test]
    fn template_malformed_yaml() {
        let f = FileReaderMock::new(
            "\
    features: title: New features
    perf:
        title: Performance improvements",
        );
        let res = Template::<Changes>::new(f);

        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .starts_with("Error parsing config YAML file:"));
    }

    #[test]
    fn template_missing_sections_key() {
        let f = FileReaderMock::new(
            "\
features:
    title: New features
perf:
    title: Performance improvements",
        );
        let res = Template::<Changes>::new(f);

        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .starts_with("Missing 'sections' key in config file"));
    }

    #[test]
    fn template_misspelled_sections_key() {
        let f = FileReaderMock::new(
            "\
sekciones:
    features:
        title: New features
    perf:
        title: Performance improvements",
        );

        let res = Template::<Changes>::new(f);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Missing 'sections' key in config file"
        );
    }

    #[test]
    fn template_malformed_sections_key() {
        let f = FileReaderMock::new(
            "\
sections: [whatever]
",
        );

        let res = Template::<Changes>::new(f);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Malformed 'sections' key in config file"
        );
    }

    #[test]
    fn template_missing_title_in_section() {
        let f = FileReaderMock::new(
            "\
sections:
    features:
        description: New features
    perf:
        title: Performance improvements",
        );

        let res = Template::<Changes>::new(f);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Missing 'title' in section 'features' in config file"
        );
    }

    #[test]
    fn template_invalid_title_in_section() {
        let f = FileReaderMock::new(
            "\
sections:
    features:
        title: New features
    perf:
        title: [Performance improvements]",
        );

        let res = Template::<Changes>::new(f);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Invalid 'title' in section 'perf' in config file"
        );
    }

    #[test]
    fn generate_commit_template_with_projects() {
        let f = FileReaderMock::new(YAML_TEMPLATE);

        let res = Template::<Changes>::new(f);
        assert!(res.is_ok());
        let template = res.unwrap();
        let stdio = FileReaderMock::new(
            "\
.githooks/commit-msg
README.md
commit.txt",
        );

        let output = template.generate_commit_template(stdio).unwrap();

        let exp_output = r"

changelog:
  project: main
  section:
  inherit: all
#
# Valid changelog sections:
#
# * security.vuln_fixes  Fixed vulnerabilities
# * features             New features
# * bug_fixes            Fixed bugs
# * breaking             Breaking changes
# * perf                 Performance improvements
# * dev                  Development";

        assert!(output.contains("project: main"));
        assert_eq!(exp_output, output);
    }

    #[test]
    fn generate_commit_template_with_changed_files_from_different_projects() {
        let f = FileReaderMock::new(YAML_TEMPLATE);

        let res = Template::<Changes>::new(f);
        assert!(res.is_ok());
        let template = res.unwrap();
        let stdio = FileReaderMock::new(
            "\
.githooks/commit-msg
README.md
mkchlog-action/README.md
commit.txt",
        );

        let output = template.generate_commit_template(stdio).unwrap();

        let exp_output = r"

changelog:
 - project:
    name: main
    section:
    inherit: all
 - project:
    name: mkchlog-action
    section:
    inherit: all
#
# Valid changelog sections:
#
# * security.vuln_fixes  Fixed vulnerabilities
# * features             New features
# * bug_fixes            Fixed bugs
# * breaking             Breaking changes
# * perf                 Performance improvements
# * dev                  Development";

        assert_eq!(exp_output, output);
    }

    #[test]
    fn generate_commit_template_fails_when_could_not_determine_project() {
        let f = FileReaderMock::new(YAML_TEMPLATE);

        let res = Template::<Changes>::new(f);
        assert!(res.is_ok());
        let template = res.unwrap();
        let stdio = FileReaderMock::new(
            "\
.githooks/commit-msg
README.md
some_new_dir/README.md
commit.txt",
        );

        let res = template.generate_commit_template(stdio);

        assert!(res.is_err());
        assert!(res
        .unwrap_err()
        .to_string()
        .starts_with("Could not determine project for file: 'some_new_dir/README.md'. Is the directory correctly set in the config file?"));
    }

    #[test]
    fn generate_commit_template_without_projects() {
        let f = FileReaderMock::new(
            "\
sections:
    security:
        title: Security
        description: This section contains very important security-related changes.
        subsections:
            vuln_fixes:
                title: Fixed vulnerabilities
    features:
        title: New features
    bug_fixes:
        title: Fixed bugs
    breaking:
        title: Breaking changes
    perf:
        title: Performance improvements
    dev:
        title: Development
        description: Internal development changes
",
        );

        let res = Template::<Changes>::new(f);
        assert!(res.is_ok());
        let template = res.unwrap();
        let stdio = FileReaderMock::new(
            "\
README.md
src/config.rs",
        );

        let output = template.generate_commit_template(stdio).unwrap();

        let exp_output = r"

changelog:
  section:
  inherit: all
#
# Valid changelog sections:
#
# * security.vuln_fixes  Fixed vulnerabilities
# * features             New features
# * bug_fixes            Fixed bugs
# * breaking             Breaking changes
# * perf                 Performance improvements
# * dev                  Development";

        assert!(!output.contains("project:"));
        assert_eq!(exp_output, output);
    }
}
