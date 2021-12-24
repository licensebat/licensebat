use serde::de::{Error, MapAccess, Visitor};
use serde::Deserializer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DartDependency {
    pub version: String,
    pub source: String,     // hosted, sdk, git
    pub dependency: String, // direct main, transitive
    #[serde(deserialize_with = "description_deserialize")]
    pub description: Description,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Description {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default, rename = "ref")]
    pub reference: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DartDependencies {
    pub packages: HashMap<String, DartDependency>,
}

impl DartDependencies {
    /// Collects all the dependencies into a a vector.
    pub fn into_vec_collection(self) -> Vec<DartDependency> {
        self.packages
            .into_iter()
            .map(|(name, mut dependency)| {
                if dependency.description.name.is_none() {
                    dependency.description.name = Some(name);
                }
                dependency
            })
            .collect()
    }
}

/// Visitor for the description field.
/// It supports &str and maps
struct DescriptionVisitor;

impl<'de> Visitor<'de> for DescriptionVisitor {
    type Value = Description;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string or map")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        // Normally, this is the case for source: sdk (flutter libraries)
        let description = Description {
            name: Some(value.to_string()),
            ..Description::default()
        };
        Ok(description)
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut description = Description::default();

        while let Some((key, value)) = map.next_entry::<String, String>()? {
            match key.as_ref() {
                "path" => description.path = Some(value),
                "name" => description.name = Some(value),
                "ref" => description.reference = Some(value),
                "url" => description.url = Some(value),
                _ => (),
            }
        }
        Ok(description)
    }
}

/// Helper function to deserialize the description property of the [`DartDependency`] struct
fn description_deserialize<'de, D>(deserializer: D) -> Result<Description, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(DescriptionVisitor {})
}
