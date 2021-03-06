//! Exposes a struct to manage the `.licrc` file information and validate the dependencies accordingly.
//!
//! When using the `licrc-from-file` feature, a [`LicRc::from_relative_path`] associated function will be available for you to load the information from a file.
use crate::RetrievedDependency;
use serde::{Deserialize, Serialize};
use tracing::instrument;

/// Represents the `.licrc` configuration file.
/// This file is the one used in your project to define which licenses are accepted/unaccepted
/// and which dependencies should be ignored.
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
    /// Error trying to open or read the .licrc file
    #[error("Error trying to open and read the .licrc file: {0}")]
    Io(#[from] std::io::Error),
    /// Error parsing the .licrc file
    #[error("Error trying to parse the .licrc file: {0}")]
    Toml(#[from] toml::de::Error),
}

#[cfg(feature = "licrc-from-file")]
impl LicRc {
    /// Loads a .licrc from a relative path.
    /// You must compile this crate with the `licrc-from-file` feature for this to be available.
    #[instrument(skip(relative_path))]
    pub fn from_relative_path(relative_path: impl AsRef<std::path::Path>) -> Result<Self, Error> {
        let licrc_path = std::env::current_dir()?.join(relative_path);
        let licrc_content = std::fs::read_to_string(licrc_path)?;
        let licrc = toml::from_str(&licrc_content)?;
        Ok(licrc)
    }
}

impl LicRc {
    /// Validates a specific [`RetrievedDependency`].
    /// Note that it will set the dependency's `validated` property to `true`.
    /// While checking it's validaty against what's been declared in the `.licrc` file it can also modify `is_ignored` and `is_valid` properties.
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
            for lic in &licenses {
                if let Some(accepted) = self.licenses.accepted.as_ref() {
                    if !accepted.contains(lic) {
                        make_invalid(dependency, lic);
                    }
                } else if let Some(unaccepted) = self.licenses.unaccepted.as_ref() {
                    if unaccepted.contains(lic) {
                        make_invalid(dependency, lic);
                    }
                }
            }
        } else {
            tracing::error!("Licenses are None!! At this point, this shouldn't happen. Check out the dependency validation logic");
        }
    }
}

/// Marks a dependency as invalid and if it doesnt' have any error it adds one saying `Not compliant`.
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
    /// List of accepted licenses (see <https://spdx.org/licenses/>)
    pub accepted: Option<Vec<String>>,
    /// List of unaccepted licenses (see <https://spdx.org/licenses/>)
    pub unaccepted: Option<Vec<String>>,
}

/// Holds information about dependency specifics.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct LicRcDependencies {
    /// List of ignored dependencies.
    /// These dependencies won't be validated.
    /// You must use the name of the dependency here.
    pub ignored: Option<Vec<String>>,
}

/// Holds information about the behavior of the validation process.
/// **This only applies for the [GITHUB API integrated project](https://github.com/marketplace/licensebat)**.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct LicRcBehavior {
    /// If set to false Licensebat will validate the dependencies no matter what file has been modified.
    /// If set to true, validation will only happen when one of the dependency files or the .licrc files has been modified in the commit.
    pub run_only_on_dependency_modification: Option<bool>,
    /// If set to true, Licensebat will execute the check but it won't block the PR.
    pub do_not_block_pr: Option<bool>,
}
