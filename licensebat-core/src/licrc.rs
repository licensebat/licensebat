use crate::RetrievedDependency;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::instrument;

/// Represents the Privateer configuration file.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct LicRc {
    /// List of accepted and unaccepted licenses.
    pub licenses: LicRcLicenses,
    /// List of ignored dependencies and dependency settings.
    pub dependencies: LicRcDependencies,
    /// Properties that affect the behavior of the validation.
    pub behavior: LicRcBehavior,
}

/// Error raised by a collector while parsing/getting the dependencies.
#[cfg(feature = "licrc-from-file")]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error trying to open and read the .licrc file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error trying to parse the .licrc file: {0}")]
    Toml(#[from] toml::de::Error),
}

#[cfg(feature = "licrc-from-file")]
impl LicRc {
    #[instrument(skip(relative_path))]
    pub fn from_relative_path(relative_path: impl AsRef<Path>) -> Result<Self, Error> {
        let licrc_path = std::env::current_dir()?.join(relative_path);
        let licrc_content = std::fs::read_to_string(licrc_path)?;
        let licrc = toml::from_str(&licrc_content)?;
        Ok(licrc)
    }
}

impl LicRc {
    #[instrument(skip(self))]
    pub fn validate(&self, dependency: &mut RetrievedDependency) {
        dependency.validated = true;
        if self
            .dependencies
            .ignored
            .as_ref()
            .unwrap_or(&vec![])
            .contains(&dependency.name)
        {
            dependency.is_ignored = true;
            tracing::debug!(dependency = ?dependency, "Dependency has been ignored");
            return;
        }

        if !dependency.is_valid {
            tracing::debug!(dependency = ?dependency, "Dependency is invalid");
            return;
        }

        if let Some(licenses) = dependency.licenses.clone() {
            licenses.iter().for_each(|lic| {
                if let Some(accepted) = self.licenses.accepted.as_ref() {
                    if !accepted.contains(lic) {
                        make_invalid(dependency, lic);
                    }
                } else if let Some(unaccepted) = self.licenses.unaccepted.as_ref() {
                    if unaccepted.contains(lic) {
                        make_invalid(dependency, lic);
                    }
                }
            });
        } else {
            tracing::error!("Licenses are None!! At this point, this shouldn't happen. Check out the dependency validation logic");
        }
    }
}

#[instrument]
fn make_invalid(dependency: &mut RetrievedDependency, license: &str) {
    tracing::debug!(
        ?dependency,
        license,
        "No compliant dependency marked as invalid"
    );
    dependency.is_valid = false;
    // preserve pre-existing error
    if dependency.error.is_none() {
        dependency.error = Some("Not compliant".to_string());
    }
}

/// Holds information about the accepted or unaccepted licenses.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct LicRcLicenses {
    pub accepted: Option<Vec<String>>,
    pub unaccepted: Option<Vec<String>>,
}

/// Holds information about dependency specifics.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct LicRcDependencies {
    /// List of ignored dependencies.
    /// These dependencies won't be validated.
    pub ignored: Option<Vec<String>>,
}

/// Holds information about the behavior of the validation process.
/// This only applies for the GITHUB API integrated project.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct LicRcBehavior {
    /// If set to false Privateer will validate the dependencies no matter what
    /// file has been modified. If set to true, validation will only
    /// happen when one of the dependency files or the .licrc files has been modified in the commit.
    pub run_only_on_dependency_modification: Option<bool>,
    /// If set to true, Privateer will execute the check but it won't block the PR
    pub do_not_block_pr: Option<bool>,
}
