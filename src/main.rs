use clap::Parser;

mod args;
mod build;
mod init;
mod interface;
mod manifest;

use args::Cli;
use interface::Interface;

fn main() -> anyhow::Result<()> {
    let (command, interface) = Cli::parse().split_interface();
    let mut interface = Interface::try_from(interface)?;

    let result = match command {
        args::Subcommands::Init(v) => init::init(v, &mut interface),
        args::Subcommands::Build(v) => build::build(v, &mut interface),
    };

    if let Err(err) = result {
        interface.bail(err.to_string());
    }

    interface.info("Done.");

    Ok(())
}
