use std::{
    fs,
    io::{self, Write},
    path::Path,
    process::Command,
};

use clap::Parser;

mod args;
mod definitions;
mod interface;
mod manifest;

use args::Cli;
use definitions::Definitions;
use interface::Interface;
use manifest::Dependencies;

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let mut interface = Interface::default();
    let manifest = interface.manifest(&args)?;
    let target = interface.target_dir(&args, &manifest)?;
    let definitions = interface.definitions(&args, &manifest)?;

    git_clone(&mut interface, &args, &target)?;
    generate_wasm_project(&manifest, &target, &definitions)?;
    generate_snap(&target)?;

    println!(
        "Snap generated on `{}`",
        target.join("packages").join("snap").display()
    );

    println!(
        "To run the web development server, install Metamask Flask and run `yarn start` on `{}`",
        target.display()
    );

    Ok(())
}

fn git_clone<P>(interface: &mut Interface, args: &Cli, target: P) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let target = target.as_ref();
    println!("Cloning into directory `{}`...", target.display());

    let output = interface.git_clone(&args, target)?;
    io::stderr().write_all(&output.stderr)?;
    io::stdout().write_all(&output.stdout)?;
    if !output.status.success() {
        anyhow::bail!("Git clone failed; did you forget to install git?");
    }

    Ok(())
}

fn generate_wasm_project<P>(
    manifest: &manifest::Manifest,
    target: P,
    definitions: &Definitions,
) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let target_dir = target.as_ref().join("external").join("sov-wasm");
    let target_manifest = target_dir.join("Cargo.toml");

    println!("Writing manifest to `{}`...", target_manifest.display());

    let mut output = r#"[package]
name = "sov-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
"#
    .to_string();

    if let Dependencies::Resolved {
        base,
        borsh,
        serde_json,
        sov_modules_api,
    } = &manifest.dependencies
    {
        output.push_str(&format!("{} = {}\n", manifest.name, base));
        output.push_str(&format!("borsh = {}\n", borsh));
        output.push_str(&format!("serde_json = {}\n", serde_json));
        output.push_str(&format!("sov-modules-api = {}\n", sov_modules_api));
    }

    fs::write(target_manifest, output.as_bytes())?;

    println!("Writing definitions...");

    let target_definitions = target_dir.join("src").join("definitions.rs");
    let mut output = String::new();
    output.push_str(&format!("pub type Context = {};\n", definitions.context));
    output.push_str(&format!("pub type DaSpec = {};\n", definitions.da_spec));
    output.push_str(&format!(
        "pub type RuntimeCall = {};\n",
        definitions.runtime
    ));

    fs::write(target_definitions, output.as_bytes())?;

    Ok(())
}

fn generate_snap<P>(target: P) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let target = target.as_ref();

    println!("Installing yarn dependencies on `{}`...", target.display());

    let output = Command::new("yarn")
        .arg("install")
        .current_dir(target)
        .output()?;
    io::stderr().write_all(&output.stderr)?;
    io::stdout().write_all(&output.stdout)?;
    if !output.status.success() {
        anyhow::bail!("Yarn command failed; did you forget to install yarn?");
    }

    println!("Installing yarn WASM file on `{}`...", target.display());

    let output = Command::new("yarn")
        .arg("update-wasm")
        .current_dir(target)
        .output()?;
    io::stderr().write_all(&output.stderr)?;
    io::stdout().write_all(&output.stdout)?;
    if !output.status.success() {
        anyhow::bail!("Yarn command failed; did you forget to install cargo, binaryen, or wabt?");
    }

    Ok(())
}
