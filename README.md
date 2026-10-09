# ConjoinedAPI for PumpkinMC

Rust library crate for PumpkinMC plugin developers. This initial API is independent of
Pumpkin's evolving plugin ABI: it can be called from a native Rust plugin using Tokio,
but is not a loadable Pumpkin plugin and does not claim a tested WASM/WIT integration.

## Build

Rust 1.85+ is required. Run `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`, and `cargo test --locked`.
The lockfile is committed for repeatable CI. No plugin server dependency is needed.

## Usage

Add this crate as a path dependency (or pin a Git commit when the repository is published):

```toml
[dependencies]
conjoined-api = { path = "../ConjoinedAPI_pump" }
```

Inside an existing Tokio async context:

```rust,ignore
use conjoined_api::ConjoinedClient;

let token = std::env::var("CONJOINED_TOKEN_EXAMPLE")?;
let client = ConjoinedClient::new("https://db.example.com:8443", "example", &token)?;
client.put(None, "settings", b"value".to_vec()).await?;
let value = client.get(None, "settings").await?;
// Use Some(player_uuid) for player-based values.
```

Reuse or clone the client; clones share the connection pool and 64-request concurrency limit.
Requests have a 5-second connection timeout and 10-second total timeout. Certificates are
verified using Rustls public CA roots. Redirects and plaintext HTTP are rejected.
Response bodies and values are limited to 64 KiB. Serialization belongs to the consuming plugin.
Do not block Pumpkin tick threads and do not expose credentials to untrusted code.

Read [the protocol and consistency limits](docs/PROTOCOL.md). These tests validate the client
foundation; live Pumpkin integration must be verified against your chosen server revision.
Licensed under [GPL-3.0-only](LICENSE).

Platform reference: [PumpkinMC](https://github.com/Pumpkin-MC/Pumpkin).
