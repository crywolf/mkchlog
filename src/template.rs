//! Template represents parsed YAML config file
use indexmap::IndexMap;
use std::error::Error;
use std::fs;

/// Template represents parsed YAML config file
#[derive(Debug)]
pub struct Template {
    changelog_map: ChangelogMap,
}

type ChangelogMap = IndexMap<String, Section>;
type Yaml = serde_yaml::Value;

impl Template {
    /// Parses the config (template) YAML file and returns the initialized template object.
    pub fn new(file_path: std::path::PathBuf) -> Result<Self, Box<dyn Error>> {
        let config_yml = match fs::read_to_string(&file_path) {
            Ok(config) => config,
            Err(err) => {
                return Err(format!(
                    "Error reading config YAML file '{}': {}",
                    file_path.display(),
                    err
                )
                .into())
            }
        };

        let config: Yaml = match serde_yaml::from_str(&config_yml) {
            Ok(config) => config,
            Err(err) => return Err(format!("Error parsing config YAML file: {}", err).into()),
        };

        let mut template = Self {
            changelog_map: ChangelogMap::new(),
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
#[derive(Debug, Clone)]
pub struct Section {
    pub title: String,
    pub description: String,
    pub subsections: IndexMap<String, Section>,
    pub changes: String,
}

#[cfg(test)]
mod tests {
    use super::Template;
    use std::path::PathBuf;

    #[test]
    fn template_new_with_wrong_path() {
        let res = Template::new(PathBuf::from("nonexistent.file"));
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .starts_with("Error reading config YAML file 'nonexistent.file'"));
    }
}
