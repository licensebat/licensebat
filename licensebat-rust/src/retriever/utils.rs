use licensebat_core::{Comment, Dependency, RetrievedDependency};
use tracing::instrument;

#[instrument(level = "debug")]
pub fn build_crates_io_retrieved_dependency(
    dependency: &Dependency,
    licenses: Option<Vec<String>>,
    error: Option<&str>,
    comment: Option<String>,
) -> RetrievedDependency {
    let url = format!(
        "https://crates.io/crates/{}/{}",
        dependency.name, dependency.version
    );

    let has_licenses = licenses.is_some();

    RetrievedDependency {
        name: dependency.name.clone(),
        version: dependency.version.clone(),
        url: Some(url),
        dependency_type: crate::RUST.to_owned(),
        validated: false,
        is_valid: has_licenses && error.is_none(),
        is_ignored: false,
        error: if let Some(err) = error {
            Some(err.to_string())
        } else if has_licenses {
            None
        } else {
            Some("No License".to_owned())
        },
        licenses: if has_licenses {
            licenses
        } else {
            Some(vec!["NO-LICENSE".to_string()])
        },
        comment: if let Some(comment) = comment {
            Some(Comment::non_removable(comment))
        } else if has_licenses {
            None
        } else {
            Some(Comment::removable("Consider **ignoring** this specific dependency. You can also accept the **NO-LICENSE** key to avoid these issues."))
        },
    }
}
