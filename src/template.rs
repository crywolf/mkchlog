use std::fs;

#[derive(Debug)]
pub struct Template {
    data: Data,
}

type Data = serde_yaml::Value;

impl Template {
    pub fn new(file_path: std::path::PathBuf) -> Result<Self, String> {
        let config_yml = match fs::read_to_string(&file_path) {
            Ok(config) => config,
            Err(err) => {
                return Err(format!(
                    "Error reading config YAML file '{}': {}",
                    file_path.display(),
                    err
                ))
            }
        };

        let config: serde_yaml::Value = match serde_yaml::from_str(&config_yml) {
            Ok(config) => config,
            Err(err) => return Err(format!("Error parsing config YAML file: {}", err)),
        };

        Ok(Template { data: config })
    }

    pub fn data(&self) -> &Data {
        &self.data
    }
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
            .starts_with("Error reading config YAML file 'nonexistent.file'"));
    }
}
