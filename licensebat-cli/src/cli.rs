use structopt::StructOpt;

/// Struct representing the args of the CLI.
#[derive(Debug, StructOpt, Clone)]
#[structopt(
    name = "🦇  Licensebat",
    author("💻  Roberto Huertas <roberto.huertas@outlook.com>"),
    long_about = "🧰  Utility to help you check that your project's dependencies comply with your license policy.\n🦀  Humbly written with Rust. 🧡\n  
    ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄
    ██░█████▄██▀▄▀█░▄▄█░▄▄▀█░▄▄█░▄▄█░▄▄▀█░▄▄▀█▄░▄
    ██░█████░▄█░█▀█░▄▄█░██░█▄▄▀█░▄▄█░▄▄▀█░▀▀░██░█
    ██░▀▀░█▄▄▄██▄██▄▄▄█▄██▄█▄▄▄█▄▄▄█▄▄▄▄█▄██▄██▄█
    ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀                                       
"
)]
pub struct Cli {
    /// Path to the file containing the dependencies of the project.
    /// i.e. package-lock.json for npm projects, yarn.lock for yarn projects, etc.
    #[structopt(short, long)]
    pub dependency_file: String,
    /// Path to the .licrc file
    #[structopt(short, long, default_value = ".licrc")]
    pub licrc_file: String,
}
