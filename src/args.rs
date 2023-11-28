use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(
        long,
        short = 'q',
        action = clap::ArgAction::Count,
        global = true,
        help = "Sets the verbosity level.",
    )]
    pub quiet: u8,

    /// Defaults all inputs.
    #[arg(long, global = true)]
    pub defaults: bool,

    /// Skips all confirmations.
    #[arg(short, long, global = true)]
    pub force: bool,
}

pub struct InterfaceArgs {
    pub quiet: u8,
    pub defaults: bool,
    pub force: bool,
}

impl Cli {
    pub fn split_interface(self) -> (Subcommands, InterfaceArgs) {
        let command = match self.command {
            Commands::SovSnapGenerator { command } => command,
        };

        (
            command,
            InterfaceArgs {
                quiet: self.quiet,
                defaults: self.defaults,
                force: self.force,
            },
        )
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    SovSnapGenerator {
        #[command(subcommand)]
        command: Subcommands,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum Subcommands {
    /// Initializes the project from the provided path.
    Init(super::init::Init),

    /// Builds a project initialized via `Init`.
    Build(super::build::Build),
}
