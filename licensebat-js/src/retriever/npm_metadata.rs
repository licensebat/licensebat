use serde::de::{Error, MapAccess, SeqAccess, Visitor};
use serde::Deserializer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Holds information about dependency metadata coming from the npm registry.
#[derive(Serialize, Deserialize, Debug)]
pub struct NpmMetadata {
    pub name: String,
    pub version: String,
    #[serde(deserialize_with = "license_deserialize")]
    #[serde(default)]
    pub license: Option<String>,
    #[serde(deserialize_with = "licenses_deserialize")]
    #[serde(default)]
    pub licenses: Option<Vec<String>>,
}

impl NpmMetadata {
    /// Returns the license/s of the dependency.
    #[must_use]
    pub fn get_licenses(self) -> Option<Vec<String>> {
        if self.licenses.is_some() {
            self.licenses
        } else {
            self.license.map(|lic| vec![lic.replace("\"", "")])
        }
    }
}

/// Visitor for the licenses field.
/// It supports &str and maps
struct LicensesVisitor;

impl<'de> Visitor<'de> for LicensesVisitor {
    type Value = Option<String>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string or map")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Some(value.to_string()))
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut license = "".to_string();
        while let Some((key, value)) = map.next_entry::<String, String>()? {
            if key == "type" {
                license = value;
            }
        }
        Ok(Some(license))
    }
}

/// Visitor for the license field.
/// It supports &str, sequences and maps
struct LicenseVisitor;

impl<'de> Visitor<'de> for LicenseVisitor {
    type Value = Option<Vec<String>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string or map or sequence of map")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Some(vec![value.to_string()]))
    }

    fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
    where
        S: SeqAccess<'de>,
    {
        let mut licenses: Vec<String> = vec![];

        let license_map: HashMap<String, String> = seq
            .next_element()?
            .ok_or_else(|| Error::custom("no values in seq when looking for a license"))?;
        licenses.push(license_map["type"].clone());

        while let Some(value) = seq.next_element::<HashMap<String, String>>()? {
            licenses.push(value["type"].clone());
        }
        Ok(Some(licenses))
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut license = "".to_string();
        while let Some((key, value)) = map.next_entry::<String, String>()? {
            if key == "type" {
                license = value;
            }
        }
        Ok(Some(vec![license]))
    }
}

/// Helper function to deserialize the licenses property of the [`NpmDependency`] struct
fn licenses_deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(LicenseVisitor {})
}

fn license_deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(LicensesVisitor {})
}
