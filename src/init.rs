use std::{env, fs, path::PathBuf};

use clap::Parser;

use super::{
    interface::Interface,
    manifest::{Dependency, Manifest},
};

#[derive(Debug, Clone, Parser)]
pub struct Init {
    /// Path of the runtime module cargo project.
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Target directory to output the generated project.
    #[arg(short, long)]
    target: Option<PathBuf>,

    /// Git remote to use when cloning the origin repository.
    #[arg(short, long)]
    origin: Option<String>,

    /// Branch to use when cloning the origin repository.
    #[arg(short, long)]
    branch: Option<String>,
}

pub fn init(args: Init, interface: &mut Interface) -> anyhow::Result<()> {
    let Init {
        path,
        target,
        origin,
        branch,
    } = args;

    let cwd = env::current_dir()?;
    interface.prompt("Insert the path to your `Cargo.toml`");
    let path = interface.path_or_read(Some(&cwd.display().to_string()), path);
    let path = path
        .is_dir()
        .then(|| path.join("Cargo.toml"))
        .unwrap_or(path);

    if !path.exists() {
        anyhow::bail!(
            "Failed to locate `Cargo.toml`; {} does not exist",
            path.display()
        );
    }

    if !path.is_file() {
        anyhow::bail!(
            "Failed to locate `Cargo.toml`; {} is not a file",
            path.display()
        );
    }

    let path = path.canonicalize()?;

    interface.info(format!("Using manifest `{}`...", path.display()));

    let manifest = Manifest::read(&path, interface)?;

    interface.prompt("Insert the target directory of the project");
    let target_default = cwd
        .parent()
        .unwrap_or_else(|| cwd.as_path())
        .join(format!("{}-snap", manifest.project.name))
        .display()
        .to_string();

    let target = interface.path_or_read(Some(&target_default), target);
    if target.is_file() {
        interface.bail(format!(
            "The provided target `{}` is a file; use a directory",
            target.display()
        ));
    }

    if target.exists() {
        if fs::remove_dir(&target).is_err() {
            interface.prompt(format!(
                "The target directory `{}` already exists; overwrite? [y/n]",
                target.display()
            ));

            interface.read_confirmation();

            fs::remove_dir_all(&target)?;
        }
    }

    interface.prompt("Insert the origin git repository of the snap template");
    let origin_default = "https://github.com/Sovereign-Labs/sov-snap";
    let origin = interface.line_or_read(Some(&origin_default), origin);

    interface.prompt("Insert the branch of the snap template");
    let branch_default = "v0.1.3";
    let branch = interface.line_or_read(Some(&branch_default), branch);

    interface.info(format!(
        "Cloning the snap template into `{}`...",
        target.display()
    ));

    duct::cmd!(
        "git",
        "clone",
        "--quiet",
        "--progress",
        "-c",
        "advice.detachedHead=false",
        "--branch",
        branch,
        "--single-branch",
        "--depth",
        "1",
        origin,
        &target,
    )
    .run()?;

    interface.info(format!(
        "Cloned the snap template into `{}`",
        target.display()
    ));

    let target_wasm = target.join("external").join("sov-wasm").canonicalize()?;
    let target_wasm_manifest = target_wasm.join("Cargo.toml");
    let target_definitions = target_wasm.join("src").join("definitions.rs");

    let borsh = manifest
        .dependencies
        .get(&Dependency::new("borsh"))
        .cloned()
        .unwrap_or_else(|| String::from("\"0.10.3\""));
    let serde_json = manifest
        .dependencies
        .get(&Dependency::new("serde_json"))
        .cloned()
        .unwrap_or_else(|| String::from("\"1.0\""));
    let sov_modules_api = manifest.dependencies.get(&Dependency::new("sov-modules-api")).cloned().unwrap_or_else(|| String::from(r#"{ git = "https://github.com/Sovereign-Labs/sovereign-sdk.git", rev = "df169be", features = ["serde"] }"#));
    let sov_mock_da = manifest.dependencies.get(&Dependency::new("sov-mock-da")).cloned().unwrap_or_else(|| String::from(r#"{ git = "https://github.com/Sovereign-Labs/sovereign-sdk.git", rev = "df169be" }"#));

    let project = path
        .parent()
        .unwrap_or_else(|| path.as_path())
        .display()
        .to_string();
    let wasm_manifest = format!(
        r#"[package]
name = "sov-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
{} = {{ path = "{}" }}
borsh = {}
serde_json = {}
sov-modules-api = {}
sov-mock-da = {}
"#,
        manifest.project.name, project, borsh, serde_json, sov_modules_api, sov_mock_da
    );

    let definitions = format!(
        r#"pub type Context = sov_modules_api::default_context::ZkDefaultContext;
pub type DaSpec = sov_mock_da::MockDaSpec;
pub type RuntimeCall = {}::RuntimeCall<Context, DaSpec>;
"#,
        manifest.project.formatted
    );

    fs::write(&target_wasm_manifest, wasm_manifest)?;
    fs::write(&target_definitions, definitions)?;

    interface.info(format!(
        "Generated the snap template into `{}`",
        target.display()
    ));

    interface.info(format!(
        "Generated the WASM template into `{}`",
        target_wasm.display()
    ));

    interface.info(format!(
        "Edit the generated WASM template and run `cargo sov-snap-generator build --path {}`",
        target.display(),
    ));

    Ok(())
}
