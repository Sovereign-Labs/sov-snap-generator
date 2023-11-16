# sov-snap-generator

This utility creates [Metamask Snaps](https://metamask.io/snaps/) for Sovereign SDK modules.

## Requirements

- [Git](https://git-scm.com/)
- [Rust](https://www.rust-lang.org/tools/install)
- Rust WASI target: `rustup target add wasm32-wasi`
- [Yarn](https://yarnpkg.com/)
- [binaryen](https://github.com/WebAssembly/binaryen)
- [wabt](https://github.com/WebAssembly/wabt)
- [Metamask Flask](https://metamask.io/flask/) (optional for development environment)

## Installation

```bash
cargo install --git https://github.com/Sovereign-Labs/sov-snap-generator --tag "v0.1.0"
```

## Usage

#### Example module

This example will re-export the `RuntimeCall` from `demo-stf`.

First, we create the project:

```bash
cargo new --lib sov-runtime
cd sov-runtime
```

Next, update the `Cargo.toml` with the following:

```toml
[package]
name = "sov-runtime"
version = "0.1.0"
edition = "2021"

[dependencies]

## Required dependencies for the Snap
borsh = "0.10.3"
serde_json = "1.0"
sov-modules-api = { git = "https://github.com/Sovereign-Labs/sovereign-sdk.git", rev = "df169be", features = ["serde"] }

## Example definition of a module `RuntimeCall`
## Will be replaced by the user module implementation
demo-stf = { git = "https://github.com/Sovereign-Labs/sovereign-sdk.git", rev = "df169be", features = ["serde"] }
sov-mock-da = { git = "https://github.com/Sovereign-Labs/sovereign-sdk.git", rev = "df169be" }
```

Then, fetch the default `constants.json` required for module compilation.

```bash
wget https://raw.githubusercontent.com/Sovereign-Labs/sovereign-sdk/d42e289f26b9824b5ed54dbfbda94007dee305b2/constants.json
```

Finally, update the `src/lib.rs` with the following:

```rust
/// The `Context` will be used to define the asymmetric key pair.
pub use sov_modules_api::default_context::ZkDefaultContext as Context;

/// The `DaSpec` will be used to define the runtime specification.
pub use sov_mock_da::MockDaSpec as DaSpec;

/// The `RuntimeCall` will be the call message of the transaction to be signed. This is normally generated automatically by the SDK via the `DispatchCall` derive macro.
pub use demo_stf::runtime::RuntimeCall;
```

The utility defaults to searching for `Context`, `DaSpec`, and `RuntimeCall` definitions at the project root. However, these can be replaced with other paths.

For a sanity check, run the following:

```bash
cargo check
```

#### Generate the Snap

Some prompts can be specified through CLI arguments. For more information, run:

```bash
sov-snap-generator --help
```

To skip all prompts and checks, run:

```bash
sov-snap-generator --defaults --force
```

To run the interactive mode, at the project root, execute:

```bash
sov-snap-generator
```

The first prompt will inquire about the project `path`. It defaults to the current directory, so you can simply press `Enter`.

```bash
Insert the path to your `Cargo.toml`
> /home/sovereign/sov-runtime
```

The next prompt asks for the manifest definition to use the module as a dependency. It defaults to the parent directory of the resolved project manifest file.

The generated WASM file requires certain dependencies. The subsequent prompts will default to the dependencies specified in the project manifest file, if present. It will then ask for the following items in order:

- Base project (usually a path pointing to the parent directory of the project manifest file).
- [borsh](https://crates.io/crates/borsh)
- [serde_json](https://crates.io/crates/serde_json)
- [sov-modules-api](https://github.com/Sovereign-Labs/sovereign-sdk/tree/d42e289f26b9824b5ed54dbfbda94007dee305b2/module-system/sov-modules-api)

The next step is to define the target directory in which the generated Snap will be placed. It defaults to a new directory adjacent to the current project, suffixed by `-snap`.

The WASM file depends on specific definitions, typically customized by the module implementation. By default, it searches the root of the project for exports of `Context`, `DaSpec`, and `RuntimeCall`. The subsequent prompts will ask for these paths. If your implementation diverges from this standard, replace these items with their fully qualified paths.

The template snap will be downloaded from a GitHub release. The next prompts inquire about the repository origin and its branch/tag.

After the installation is executed, you should see a message indicating the project generated on `<TARGET>`.

#### Run a local development environment

Requirements: [Metamask Flask](https://metamask.io/flask/)

To initiate a web development environment with your Snap, execute the following:

```bash
cd ../sov-runtime-snap
yarn start
```

Your Snap will be accessible by default at http://localhost:8000. Click Connect/Reconnect to load the Snap into your Metamask Flask, enabling you to sign transactions.

This development environment allows you to submit a signed transaction to a [sov-sequencer](https://github.com/Sovereign-Labs/sovereign-sdk/tree/d42e289f26b9824b5ed54dbfbda94007dee305b2/full-node/sov-sequencer). However, most modern browsers query external services for a [CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) policy.

You can either disable CORS in your browser or set up a proxy to handle CORS requests, forwarding the payload to the sequencer.
