use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Generic dependency.
/// Language agnostic, just holds the name and the version.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Dependency {
    /// Dependency name
    pub name: String,
    /// Dependency version
    pub version: String,
}

/// A dependency that has been retrieved.
/// It holds information about licenses, errors while validating...
#[derive(Serialize, Deserialize, Debug, Eq, Ord, PartialEq, PartialOrd, Clone)]
pub struct RetrievedDependency {
    /// Dependency name.
    pub name: String,
    /// Dependency version.
    pub version: String,
    /// Dependency type (npm, dart, rust, go, python...)
    pub dependency_type: String,
    /// Url of the dependency if available.
    pub url: Option<String>,
    /// List of licenses of the dependency.
    pub licenses: Option<Vec<String>>,
    /// Indicates if the license is valid for our project or not according to our .licrc configuration file.
    pub is_valid: bool,
    /// Indicates if the dependency has been ignored according to our .licrc configuration file.
    pub is_ignored: bool,
    /// Contains information about any error that may have happened during the validation process.
    pub error: Option<String>,
    /// Comments about the license validation process.
    pub comment: Option<Comment>,
}

/// Represents a comment.
#[derive(Serialize, Deserialize, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Clone)]
pub struct Comment {
    /// The comment.
    pub text: String,
    /// If true, the comment won't be shown if the dependency is valid.
    pub remove_when_valid: bool,
}

impl Comment {
    /// Builds a removable comment.
    pub fn removable(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            remove_when_valid: true,
        }
    }

    /// Builds a non removable comment
    pub fn non_removable(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            remove_when_valid: false,
        }
    }
}
