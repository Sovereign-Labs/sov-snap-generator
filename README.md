# sov-snap-generator

This utility creates [Metamask Snaps](https://metamask.io/snaps/) for Sovereign SDK modules.

## Requirements

- [Git](https://git-scm.com/)
- [Rust](https://www.rust-lang.org/tools/install)
- [Rust WASI](https://github.com/bytecodealliance/wasmtime/blob/183cb0f2f8b0298f0bc9fd1140aaef4a0fb0368c/docs/WASI-tutorial.md#from-rust)
- [Yarn](https://yarnpkg.com/)
- [binaryen](https://github.com/WebAssembly/binaryen)
- [wabt](https://github.com/WebAssembly/wabt)
- [Metamask Flask](https://metamask.io/flask/) (optional for development environment)

## Installation

```bash
cargo install --git https://github.com/Sovereign-Labs/sov-snap-generator --tag "v0.1.2"
```

Also, check if the `wasm32-wasi` target is installed:

```bash
rustup target list --installed | grep wasm32-wasi
```

If the command above yields no output, proceed with the target installation:

```bash
rustup target add wasm32-wasi
```

## Usage

![Demo](https://github.com/Sovereign-Labs/sov-snap-generator/assets/8730839/0a9dbd0a-cf74-452f-ad5b-77c3535bf9ba)

#### Example module

This example will re-export the `RuntimeCall` from `demo-stf`.

First, we create the sample project:

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

Then, update the `src/lib.rs` with the following:

```rust
/// The `Context` will be used to define the asymmetric key pair.
pub use sov_modules_api::default_context::ZkDefaultContext as Context;

/// The `DaSpec` will be used to define the runtime specification.
pub use sov_mock_da::MockDaSpec as DaSpec;

/// The `RuntimeCall` will be the call message of the transaction to be signed. This is normally generated automatically by the SDK via the `DispatchCall` derive macro.
pub use demo_stf::runtime::RuntimeCall;
```

Finally, fetch the default `constants.json` required for module compilation.

```bash
wget https://raw.githubusercontent.com/Sovereign-Labs/sovereign-sdk/d42e289f26b9824b5ed54dbfbda94007dee305b2/constants.json
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

We use the options `--defaults` and `--force` to skip prompt confirmations and checks.

First, we generate the target project. This will create the Snap project under `../sov-runtime-snap` (i.e. `--target` argument).

```bash
cargo sov-snap-generator init --defaults --force --target ../sov-runtime-snap
```

We can perform a sanity check on the generated WASM project. This project is editable by the user to adhere to the WASM file specification. Nevertheless, functions designated with the directive `#[no_mangle]` will be consumed by the Snap and will typically remain unchanged.

```bash
cargo check --manifest-path ../sov-runtime-snap/external/sov-wasm/Cargo.toml
```

To compile the Snap, run:

```bash
cargo sov-snap-generator build --target ../sov-runtime-snap
```

Finally, you can run the local development environment.

#### Run a local development environment

Requirements: [Metamask Flask](https://metamask.io/flask/)

To initiate a web development environment with your Snap, execute the following:

```bash
cd ../sov-runtime-snap
yarn start
```

Your Snap is accessible by default at `http://localhost:8000`. To load the Snap into your Metamask Flask and enable signing of transactions, click on Connect/Reconnect.

This development environment allows you to submit a signed transaction to a [sov-sequencer](https://github.com/Sovereign-Labs/sovereign-sdk/tree/d42e289f26b9824b5ed54dbfbda94007dee305b2/full-node/sov-sequencer). However, most modern browsers query external services for a [CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) policy. Normally, a Sequencer will be served behind a layer that handles authentication.

To bypass this issue, you can either disable CORS in your browser or set up a proxy to handle CORS requests, forwarding the payload to the sequencer. Here is a minimalistic Python script that will run a CORS proxy on port 9000, redirecting all requests to `127.0.0.1:12345`:

```python
from flask import Flask, request, jsonify
import requests

app = Flask(__name__)

# Define the target URL (the URL you want to proxy to)
TARGET_URL = "http://127.0.0.1:12345"

# Enable CORS for all routes
@app.after_request
def add_cors_headers(response):
    response.headers["Access-Control-Allow-Origin"] = "*"
    response.headers["Access-Control-Allow-Methods"] = "GET, POST, OPTIONS"
    response.headers["Access-Control-Allow-Headers"] = "Content-Type"
    return response

@app.route('/', defaults={'path': ''}, methods=['GET', 'POST', 'OPTIONS'])
@app.route('/<path:path>', methods=['GET', 'POST', 'OPTIONS'])
def proxy(path):
    target_url = f"{TARGET_URL}/{path}"
    headers = {key: value for (key, value) in request.headers if key != 'Host'}

    if request.method == 'OPTIONS':
        # Handle preflight requests
        return jsonify({'status': 'ok'})

    if request.method == 'POST':
        # Forward POST request
        response = requests.post(target_url, data=request.get_data(), headers=headers)
    else:
        # Forward GET request
        response = requests.get(target_url, headers=headers)

    # Forward the received headers and content to the client
    headers = [(key, value) for (key, value) in response.headers.items()]
    return response.content, response.status_code, headers

if __name__ == '__main__':
    app.run(port=9000)  # Change the port if needed
```
