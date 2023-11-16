use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    /// Path to the directory that contains the manifest TOML file of the project.
    #[arg(short, long)]
    pub path: Option<PathBuf>,

    /// Target directory to output the generated project.
    #[arg(short, long)]
    pub target: Option<PathBuf>,

    /// Git remote to use when cloning the origin repository.
    #[arg(short, long)]
    pub origin: Option<String>,

    /// Branch to use when cloning the origin repository.
    #[arg(short, long)]
    pub branch: Option<String>,

    /// Context definition of the runtime spec.
    #[arg(short, long)]
    pub context: Option<String>,

    /// DA definition of the runtime.
    #[arg(short, long)]
    pub da_spec: Option<String>,

    /// Runtime call definition.
    #[arg(short, long)]
    pub runtime: Option<String>,

    /// Defaults all inputs.
    #[arg(long)]
    pub defaults: bool,

    /// Skips all confirmations.
    #[arg(long)]
    pub force: bool,
}
