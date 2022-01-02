#![warn(missing_docs)]

use structopt::StructOpt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // with the cli we're not constrained to normal files
    // we can also get into the dependency directory itself and
    // traverse it.
    // This basically means we can either parse a dependency manifest
    // or some directory and that the collectors don't need a specific api. So, for each supported language a different strategy may be used.

    // The user has to provide a list of languages supported and then
    // for each language, a strategy will be used.

    dotenv::dotenv().ok();
    set_up_tracing();
    let cli = licensebat_cli::Cli::from_args();
    let deps = licensebat_cli::run(cli).await?;

    // TODO: in case of errors in the licenses we should probably exit with a non-zero code
    // and probably provide some information about the error(s).

    //  show the result in json format
    tracing::debug!("Showing results");
    let json = if cfg!(debug_assertions) {
        serde_json::to_string_pretty(&deps)
    } else {
        serde_json::to_string(&deps)
    }?;
    println!("{}", json);

    // TODO: add option to write the result to a file?

    Ok(())
}

fn set_up_tracing() {
    let tracing = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env());

    if cfg!(debug_assertions) {
        tracing.pretty().init();
    } else {
        tracing.json().init();
    }
}
