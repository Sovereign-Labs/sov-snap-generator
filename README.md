# sov-snap-generator

This utility generates [Metamask Snaps](https://metamask.io/snaps/) for Sovereign SDK module implementations.

## Requirements

- [Rust](https://www.rust-lang.org/tools/install)

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

Then, we update the `Cargo.toml` with the following:

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

Then, we fetch a default `constants.json`, required for module compilation:

```bash
wget https://raw.githubusercontent.com/Sovereign-Labs/sovereign-sdk/d42e289f26b9824b5ed54dbfbda94007dee305b2/constants.json
```

Finally, we update the `src/lib.rs` with the following:

```rust
/// The `Context` will be used to define the asymmetric key pair.
pub use sov_modules_api::default_context::ZkDefaultContext as Context;

/// The `DaSpec` will be used to define the runtime specification.
pub use sov_mock_da::MockDaSpec as DaSpec;

/// The `RuntimeCall` will be the call message of the transaction to be signed. This is normally generated automatically by the SDK via the `DispatchCall` derive macro.
pub use demo_stf::runtime::RuntimeCall;
```

The utility will look, by default, for definitions of `Context`, `DaSpec`, and `RuntimeCall` at the root of the project. They, however, can be replaced by other paths.

For a sanity check, run the following:

```bash
cargo check
```

#### Generate the Snap

Some prompts can be provided via CLI arguments. For more information, run:

```bash
sov-snap-generator --help
```

On the root of the project, run:

```bash
sov-snap-generator
```

The first prompt will ask for the `path` of the project. It defaults to the current directory, so you can simply press `Enter`.

```bash
Insert the path to your `Cargo.toml`
> /home/sovereign/sov-runtime
```

The next prompt asks for the manifest definition to use the module as dependency. It defaults to the parent directory of the resolved project manifest file.

The generated WASM file must have some dependencies. The next prompts will default to the dependency specified on the project manifest file, if present, and will ask for the following items, in order:

- Base project, usually a path pointing to the parent directory of the project manifest file.
- [borsh](https://crates.io/crates/borsh)
- [serde_json](https://crates.io/crates/serde_json)
- [sov-modules-api](https://github.com/Sovereign-Labs/sovereign-sdk/tree/d42e289f26b9824b5ed54dbfbda94007dee305b2/module-system/sov-modules-api)

The next step is to define the target directory in which the generated Snap will be placed. It defaults to a new directory, neighbor to the current project, suffixed by `-snap`.

The WASM files depends on a couple of definitions, usually customized by the module implementation. It will, by default, search the root of the project for exports of `Context`, `DaSpec`, and `RuntimeCall`. The next prompts will ask for such paths; if your implementation diverges from this standard, your just replace these items for their fully qualified paths.

The template snap will be downloaded from a github release. The next prompts queries for the repository origin and its branch/tag.

After the installation is executed, you should see a `Snap generated on <TARGET>` message.

#### Run a local development environment

Requirements: [Metamask Flask](https://metamask.io/flask/)

To start a web development environment with your Snap, run the following:

```bash
cd ../sov-runtime-snap
yarn start
```

Your Snap will be available, by default, at `http://localhost:8000`. Click Connect/Reconnect to load the Snap into your Metamask Flask, and you can sign messages.

This development environment will provide the possibility to submit a signed transaction to a [sov-sequencer](https://github.com/Sovereign-Labs/sovereign-sdk/tree/d42e289f26b9824b5ed54dbfbda94007dee305b2/full-node/sov-sequencer). However, most modern browsers queries external services for a [CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) policy.

You can either disable CORS in your browser, or set a proxy that will handle the CORS requests and forward the payload to the sequencer.
