use serde::{Deserialize, Serialize};

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
