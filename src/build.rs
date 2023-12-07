use std::{env, path::PathBuf};

use clap::Parser;

use super::interface::Interface;

#[derive(Debug, Clone, Parser)]
pub struct Build {
    /// Target directory to output the generated project.
    #[arg(short, long)]
    target: Option<PathBuf>,
}

pub fn build(args: Build, interface: &mut Interface) -> anyhow::Result<()> {
    let Build { target } = args;

    let cwd = env::current_dir()?.display().to_string();

    interface.prompt("Insert the target directory of the project");

    let target = interface.path_or_read(Some(&cwd), target);
    if target.is_file() {
        interface.bail(format!(
            "The provided target `{}` is a file; use a directory",
            target.display()
        ));
    }

    interface.info(format!("Using target root `{}`...", target.display()));

    duct::cmd!("yarn", "install").dir(&target).run()?;

    interface.info(format!(
        "Yarn packages installed on `{}`...",
        target.display()
    ));

    duct::cmd!("yarn", "update-wasm").dir(&target).run()?;

    interface.info(format!("WASM file built on `{}`...", target.display()));

    duct::cmd!("yarn", "build").dir(&target).run()?;

    interface.info(format!("Yarn project built on `{}`...", target.display()));
    interface.info("To start the browser application, run `yarn start` on the project root.");

    Ok(())
}
