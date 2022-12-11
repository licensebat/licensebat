use licensebat_core::{Comment, Dependency, RetrievedDependency};
use tracing::instrument;

#[instrument(level = "debug")]
pub fn crates_io_retrieved_dependency(
    dependency: &Dependency,
    licenses: Option<Vec<String>>,
    error: Option<&str>,
    comment: Option<String>,
    suggested_licenses: Option<Vec<(String, f32)>>,
) -> RetrievedDependency {
    let url = format!(
        "https://crates.io/crates/{}/{}",
        dependency.name, dependency.version
    );

    RetrievedDependency::new(
        dependency.name.clone(),
        dependency.version.clone(),
        crate::RUST.to_owned(),
        Some(url),
        licenses,
        error.map(std::string::ToString::to_string),
        comment.map(Comment::removable),
        suggested_licenses,
    )
}
