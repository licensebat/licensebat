//! CLI executable logic lives here.
#![doc(html_logo_url = "https://licensebat.com/images/not_used/logo_red_ferris.png")]
#![doc(html_favicon_url = "https://licensebat.com/images/not_used/favicons_red/favicon.ico")]
#![warn(missing_docs)]

use licensebat_core::RetrievedDependency;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    set_up_tracing();
    let cli = licensebat_cli::Cli::from_args();
    let licensebat_cli::RunResult {
        licrc,
        dependencies,
    } = licensebat_cli::run(cli).await?;

    show_result(&dependencies)?;

    if licrc.behavior.do_not_block_pr.unwrap_or(false) {
        Ok(())
    } else {
        let has_invalid_dependencies = dependencies.iter().any(|d| !d.is_valid);
        if has_invalid_dependencies {
            std::process::exit(1);
        }
        Ok(())
    }
}

/// Prints the dependencies in the stdout
fn show_result(deps: &[RetrievedDependency]) -> anyhow::Result<()> {
    tracing::debug!("Showing results");
    let json = if cfg!(debug_assertions) {
        serde_json::to_string_pretty(&deps)
    } else {
        serde_json::to_string(&deps)
    }?;
    println!("{}", json);
    Ok(())
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
