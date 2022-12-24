//! CLI executable logic lives here.
#![doc(html_logo_url = "https://licensebat.com/images/not_used/logo_red_ferris.png")]
#![doc(html_favicon_url = "https://licensebat.com/images/not_used/favicons_red/favicon.ico")]
#![warn(missing_docs)]

use licensebat_cli::OutputFormat;
use licensebat_core::RetrievedDependency;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    set_up_tracing();
    let cli = licensebat_cli::Cli::from_args();
    let format = cli.output_format.clone();
    let licensebat_cli::RunResult {
        licrc,
        mut dependencies,
    } = licensebat_cli::run(cli).await?;

    let invalid_dependencies_count = dependencies
        .iter()
        .filter(|d| !d.is_valid && !d.is_ignored)
        .count();

    match format {
        OutputFormat::Json => show_result_as_json(&dependencies)?,
        OutputFormat::Markdown => {
            show_result_as_markdown(&mut dependencies, invalid_dependencies_count)
        }
    };

    if invalid_dependencies_count > 0 && !licrc.behavior.do_not_block_pr.unwrap_or(false) {
        std::process::exit(1);
    }
    Ok(())
}

/// Prints the dependencies in the stdout as json
fn show_result_as_json(deps: &[RetrievedDependency]) -> anyhow::Result<()> {
    tracing::debug!("Showing results as JSON");
    let json = if cfg!(debug_assertions) {
        serde_json::to_string_pretty(&deps)
    } else {
        serde_json::to_string(&deps)
    }?;
    println!("{}", json);
    Ok(())
}

/// Prints the dependencies in the stdout as markdown
fn show_result_as_markdown(deps: &mut [RetrievedDependency], invalid_dependencies_count: usize) {
    tracing::debug!("Showing results as MARKDOWN");
    let total = deps.len();

    let md = {
        let header =
            "| Result | Name |  Version | Type | Validity | Ignored | Licenses | Error | Comments | Is Dev | Is Optional |";
        let header_separator = "|---|---|---|---|---|---|---|---|---|---|---|";

        deps.sort_by(|d1, d2| {
            let o_name = d1.name.cmp(&d2.name);
            let o_valid = d1.is_valid.cmp(&d2.is_valid);

            match o_valid {
                std::cmp::Ordering::Equal => o_name,
                _ => o_valid,
            }
        });

        let deps_str: Vec<String> = deps
            .iter()
            .map(|dep| {
                format!(
                    "| {} | **{}** | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                    if dep.is_valid {
                        if let Some(comment) = &dep.comment {
                            if comment.remove_when_valid {
                                ":green_circle:"
                            } else {
                                ":yellow_circle:"
                            }
                        } else {
                            ":green_circle:"
                        }
                    } else if dep.is_ignored {
                        ":large_blue_circle:"
                    } else {
                        ":red_circle:"
                    },
                    match dep.url.as_ref() {
                        Some(url) => format!("[{}]({})", dep.name, url,),
                        None => dep.name.to_owned(),
                    },
                    dep.version,
                    dep.dependency_type,
                    if dep.is_valid { "" } else { "Invalid" },
                    if dep.is_ignored {
                        "Ignored by .licrc"
                    } else {
                        ""
                    },
                    dep.licenses.as_ref().unwrap_or(&vec![]).join(", "),
                    dep.error.as_ref().map_or("", |s| s.as_str()),
                    dep.comment.as_ref().map_or("", |c| {
                        if c.remove_when_valid && (dep.is_valid || dep.is_ignored) {
                            ""
                        } else {
                            c.text.as_str()
                        }
                    }),
                    dep.is_dev
                        .map(|b| if b { "True" } else { "False" })
                        .unwrap_or("-"),
                    dep.is_optional
                        .map(|b| if b { "True" } else { "False" })
                        .unwrap_or("-"),
                )
            })
            .collect();

        format!(
            "# Licensebat analysis result ({} dependencies - {} invalid)\n{}\n{}\n{}",
            total,
            invalid_dependencies_count,
            header,
            header_separator,
            deps_str.join("\n")
        )
    };

    println!("{}", md);
}

/// Sets up the tracing subscriber.
/// It will use a pretty print in debug and json in --release mode.
fn set_up_tracing() {
    let tracing = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env());

    if cfg!(debug_assertions) {
        tracing.pretty().init();
    } else {
        tracing.json().init();
    }
}
