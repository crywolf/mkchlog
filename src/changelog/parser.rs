//! YAML parser

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_yaml::Error;
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

/// Parse the content of a changelog message into a [`Changelog`] structure
pub fn parse(s: &str) -> Result<Changelog, Error> {
    let s = &format!("changelog:{}", s);
    let chw = serde_yaml::from_str::<ChangelogWrapper>(s)?;
    Ok(chw.changelog)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ChangelogWrapper {
    #[serde(deserialize_with = "string_or_struct_or_vec")]
    changelog: Changelog,
}

#[derive(Debug, Deserialize, Default, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Changelog {
    #[serde(default)]
    pub skip: bool,
    pub project: Option<String>,
    pub section: String,
    pub title: Option<String>,
    #[serde(rename = "title-is-enough", default)]
    pub title_is_enough: bool,
    pub description: Option<String>,
    pub inherit: Option<String>, // ignored, only for backwards compatibility
    #[serde(skip)]
    pub projects: Option<Vec<Project>>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ProjectWrapper {
    pub project: Project,
}

#[derive(Debug, Deserialize, Default, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Project {
    #[serde(default)]
    pub skip: bool,
    pub name: String,
    pub section: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "title-is-enough", default)]
    pub title_is_enough: bool,
    pub description: Option<String>,
}

impl From<Project> for Changelog {
    fn from(project: Project) -> Self {
        Changelog {
            skip: project.skip,
            project: Some(project.name),
            section: project.section.unwrap_or_default(),
            title: project.title,
            title_is_enough: project.title_is_enough,
            description: project.description,
            inherit: None,
            projects: None,
        }
    }
}

impl From<Vec<Project>> for Changelog {
    fn from(projects: Vec<Project>) -> Self {
        Changelog {
            projects: Some(projects),
            ..Default::default()
        }
    }
}

impl FromStr for Changelog {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim() != "skip" {
            return Err(de::Error::custom(format!("Unexpected value '{}'", s)));
        }

        let chlg = Changelog {
            skip: true,
            ..Default::default()
        };

        Ok(chlg)
    }
}

// T type can be deserialized either from a string, map or sequence of maps
fn string_or_struct_or_vec<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = Error> + From<Vec<Project>>,
    D: Deserializer<'de>,
{
    // This is a Visitor that forwards string types to T's `FromStr` impl and
    // forwards map types to T's `Deserialize` impl. The `PhantomData` is to
    // keep the compiler from complaining about T being an unused generic type
    // parameter. We need T in order to know the Value type for the Visitor
    // impl.
    struct StringOrStructOrVec<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStructOrVec<T>
    where
        T: Deserialize<'de> + FromStr<Err = Error> + From<Vec<Project>>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map or sequence")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            T::from_str(value).map_err(|err| de::Error::custom(err.to_string()))
        }

        fn visit_map<M>(self, map: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            // `MapAccessDeserializer` is a wrapper that turns a `MapAccess`
            // into a `Deserializer`, allowing it to be used as the input to T's
            // `Deserialize` implementation. T then deserializes itself using
            // the entries from the map visitor.
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<T, S::Error>
        where
            S: SeqAccess<'de>,
        {
            // Convert the deserialized sequence of ProjectWrappers into a Changelog instance
            let mut projects = Vec::<Project>::new();
            while let Some(pw) = seq.next_element::<ProjectWrapper>()? {
                projects.push(pw.project);
            }

            Ok(projects.into())
        }
    }

    deserializer.deserialize_any(StringOrStructOrVec(PhantomData))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_changelog_yaml_skip() {
        let yaml = " skip";

        let expected = Changelog {
            skip: true,
            project: None,
            section: "".to_owned(),
            title: None,
            title_is_enough: false,
            description: None,
            inherit: None,
            projects: None,
        };

        let res = parse(yaml).unwrap();
        assert_eq!(res, expected);
    }

    #[test]
    fn parse_changelog_yaml_map() {
        let yaml = "
        project: mkchlog-action
        section: doc
        title-is-enough: true";

        let expected = Changelog {
            skip: false,
            project: Some("mkchlog-action".to_owned()),
            section: "doc".to_owned(),
            title: None,
            title_is_enough: true,
            description: None,
            inherit: None,
            projects: None,
        };

        let res = parse(yaml).unwrap();
        assert_eq!(res, expected);

        let yaml = "
        section: features
        project: mkchlog";

        let expected = Changelog {
            skip: false,
            project: Some("mkchlog".to_owned()),
            section: "features".to_owned(),
            title: None,
            title_is_enough: false,
            description: None,
            inherit: None,
            projects: None,
        };

        let res = parse(yaml).unwrap();
        assert_eq!(res, expected);

        let yaml = "
        section: features
        project: ";

        let expected = Changelog {
            skip: false,
            project: None,
            section: "features".to_owned(),
            title: None,
            title_is_enough: false,
            description: None,
            inherit: None,
            projects: None,
        };

        let res = parse(yaml).unwrap();
        assert_eq!(res, expected);

        let yaml = "
        section: features";

        let expected = Changelog {
            skip: false,
            project: None,
            section: "features".to_owned(),
            title: None,
            title_is_enough: false,
            description: None,
            inherit: None,
            projects: None,
        };

        let res = parse(yaml).unwrap();
        assert_eq!(res, expected);

        let yaml = "
        section: features
        nonsense: yes";

        let res = parse(yaml);
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .starts_with("changelog: unknown field `nonsense`"));

        let yaml = "
        project: mkchlog";

        let res = parse(yaml);
        assert!(res.is_err());

        assert!(res
            .unwrap_err()
            .to_string()
            .starts_with("changelog: missing field `section`"));

        let yaml = "
        section:";

        let res = parse(yaml).unwrap();

        assert_eq!(res.section, "~");
    }

    #[test]
    fn parse_changelog_yaml_list() {
        let yaml = "
        - project:
           name: mkchlog
           section: dev
           title-is-enough: true
        - project:
           name: mkchlog-action
           skip: true";

        let expected = Changelog {
            skip: false,
            project: None,
            section: "".to_owned(),
            title: None,
            title_is_enough: false,
            description: None,
            inherit: None,
            projects: Some(vec![
                Project {
                    skip: false,
                    name: "mkchlog".to_owned(),
                    section: Some("dev".to_owned()),
                    title: None,
                    title_is_enough: true,
                    description: None,
                },
                Project {
                    skip: true,
                    name: "mkchlog-action".to_owned(),
                    section: None,
                    title: None,
                    title_is_enough: false,
                    description: None,
                },
            ]),
        };

        let res = parse(yaml).unwrap();
        assert_eq!(res, expected);
    }
}
