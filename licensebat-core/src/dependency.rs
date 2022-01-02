use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Generic and plain dependency without any extra information.
/// Language agnostic, just holds the name and the version.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, Ord, PartialEq, PartialOrd, Default)]
pub struct Dependency {
    /// Dependency name
    pub name: String,
    /// Dependency version
    pub version: String,
}

/// A dependency that has been retrieved from its source.
/// The source can be anything, from a third party API (i.e. npm, pub.dev or crates.io APIs) to the file system.
/// It holds information about licenses, errors while validating...
#[derive(Serialize, Deserialize, Debug, Eq, Ord, PartialEq, PartialOrd, Clone, Default)]
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
    /// Set to true if the dependency has been validated against the licrc.
    pub validated: bool,
    /// Indicates if the license is valid for our project or not according to our .licrc configuration file.
    pub is_valid: bool,
    /// Indicates if the dependency has been ignored according to our .licrc configuration file.
    pub is_ignored: bool,
    /// Contains information about any error that may have happened during the validation process.
    pub error: Option<String>,
    /// Comments about the license validation process.
    pub comment: Option<Comment>,
}

impl RetrievedDependency {
    /// Creates a new `RetrievedDependency` with the given parameters.
    /// Note that some properties will be automatically set depending on the other ones.
    /// For example, if the `licenses` parameter is `None`, the `is_valid` property will be set to `false`.
    /// Use the default method if you just want to create an instance with all  the defaults.
    /// This method it's intended to be used once you have retrieved the dependency from its source (i.e. npm, github, etc).
    #[must_use]
    pub fn new(
        name: String,
        version: String,
        dependency_type: String,
        url: Option<String>,
        licenses: Option<Vec<String>>,
        error: Option<String>,
        comment: Option<Comment>,
    ) -> Self {
        let has_licenses = licenses.is_some();

        Self {
            name,
            version,
            dependency_type,
            url,
            licenses: licenses.or_else(|| Some(vec!["NO-LICENSE".to_string()])),
            validated: false,
            is_valid: has_licenses && error.is_none(),
            is_ignored: false,
            error: error.or_else(|| {
                if has_licenses {
                    None
                } else {
                    Some("No License".to_owned())
                }
            }),
            comment: comment.or_else(|| {
                if has_licenses {
                    None
                } else {
                    Some(Comment::removable("Consider ignoring this specific dependency. You can also accept the NO-LICENSE key."))
                }
            }),
        }
    }
}

/// A comment to be added in a [`RetrievedDependency`] once it has been retrieved or validated.
/// It normally adds information about what went wrong.
#[derive(Serialize, Deserialize, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Clone)]
pub struct Comment {
    /// The comment text.
    pub text: String,
    /// If true, the comment won't be shown if the dependency is valid.
    pub remove_when_valid: bool,
}

impl Comment {
    /// Builds a removable comment.
    /// This basically mean it won't be shown if the dependency is flagged as valid.
    pub fn removable(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            remove_when_valid: true,
        }
    }

    /// Builds a non removable comment.
    /// This comment will be shown no matter if the dependency is valid or not.
    pub fn non_removable(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            remove_when_valid: false,
        }
    }
}
