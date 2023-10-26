//! Template represents parsed YAML config file
use indexmap::IndexMap;
use std::error::Error;
use std::io::Read;
use serde_yaml::Value;

/// Template represents parsed YAML config file
#[derive(Debug)]
pub struct Template {
    changelog_map: ChangelogMap,
    pub settings: Settings,
}

#[derive(Debug)]
/// Settings represent options that were set in YAML config file
pub struct Settings {
    pub skip_commits_up_to: Option<String>,
    pub git_path: Option<std::path::PathBuf>,
}

type ChangelogMap = IndexMap<String, Section>;
type Yaml = serde_yaml::Value;

impl Template {
    /// Parses the config (template) YAML file and returns the initialized template object.
    pub fn new(mut file: impl Read) -> Result<Self, Box<dyn Error>> {
        let mut config_yml = String::new();
        file.read_to_string(&mut config_yml)?;

        Self::from_str(&config_yml)
    }

    pub fn from_str(config_yml: &str) -> Result<Self, Box<dyn Error>> {
        let config: Yaml = match serde_yaml::from_str(config_yml) {
            Ok(config) => config,
            Err(err) => return Err(format!("Error parsing config YAML file: {}", err).into()),
        };

        let skip_commits_up_to = config
            .get("skip-commits-up-to")
            .map(|v| {
                v.as_str()
                    .map(ToOwned::to_owned)
                    .ok_or("'skip-commits-up-to' key must be a string")
            })
            .transpose()?;

        let git_path = config
            .get("git-path")
            .map(|v| {
                v.as_str()
                    .map(std::path::PathBuf::from)
                    .ok_or("'git-path' key must be a string")
            })
            .transpose()?;

        let mut template = Self {
            changelog_map: ChangelogMap::new(),
            settings: Settings {
                skip_commits_up_to,
                git_path,
            },
        };

        template.parse_config(config)?;

        Ok(template)
    }

    /// Validates template data extracted from the configuration (template) file
    /// and prepares data structure for storing changelog data
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
                changes: String::new(),
            };

            if let Some(subsections) = val.get(&Value::from("subsections")) {
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

            self.changelog_map.insert(sec.to_string(), section);
        }

        Ok(())
    }

    /// Returns data structure with initialized sections for storing changelog data.
    pub fn data(&self) -> ChangelogMap {
        self.changelog_map.clone()
    }
}

/// Data structure to store changelog section data
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub title: String,
    pub description: String,
    pub subsections: IndexMap<String, Section>,
    pub changes: String,
}

#[cfg(test)]
mod tests {
    use super::Template;
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

    #[test]
    fn template_valid_yaml() {
        use super::Section;
        use indexmap::IndexMap;

        let f = FileReaderMock::new(
            "\
skip-commits-up-to: bc58e6bf2cf640d46aa832e297d0f215f76dfce0

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
",
        );

        let res = Template::new(f);
        assert!(res.is_ok());

        let template = res.unwrap();

        // check for correctly parsed settings
        let settings = &template.settings;

        assert_eq!(
            settings.skip_commits_up_to.as_ref().unwrap(),
            "bc58e6bf2cf640d46aa832e297d0f215f76dfce0"
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
                changes: "".to_owned(),
            },
        );
        assert_eq!(
            exp_sections[0],
            Section {
                title: "Security".to_owned(),
                description: "This section contains very important security-related changes."
                    .to_owned(),
                subsections: subsecs,
                changes: "".to_owned(),
            }
        );

        // 'features' section
        assert_eq!(
            exp_sections[1],
            Section {
                title: "New features".to_owned(),
                description: "".to_owned(),
                subsections: IndexMap::new(),
                changes: "".to_owned(),
            }
        );

        // 'dev' section
        assert_eq!(
            exp_sections[5],
            Section {
                title: "Development".to_owned(),
                description: "Internal development changes".to_owned(),
                subsections: IndexMap::new(),
                changes: "".to_owned(),
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
        let res = Template::new(f);

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
        let res = Template::new(f);

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

        let res = Template::new(f);
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

        let res = Template::new(f);
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

        let res = Template::new(f);
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

        let res = Template::new(f);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Invalid 'title' in section 'perf' in config file"
        );
    }
}
