# Package a configured native CLI

A native application embeds a CLI definition and an application profile and calls `launch_configured_cli`.
The kernel handles options, contexts, completion, HTTP reads, observations, and tables.
AGP's first consumer lives in its `rustcli/` directory and contains a small Rust entry point plus two authored documents.

## Use transferred configuration

With sibling `cli` and `agp` checkouts, run this from the CLI repository:
```sh
cargo build --locked --bin cli
target/debug/cli app ../agp/rustcli/spec/definition.json \
  ../agp/rustcli/spec/application.json tree
target/debug/cli app ../agp/rustcli/spec/definition.json \
  ../agp/rustcli/spec/application.json connections --help
```

Supply a running local node's actual management URL to read it:
```sh
target/debug/cli app ../agp/rustcli/spec/definition.json \
  ../agp/rustcli/spec/application.json --url "$AGP_MANAGEMENT_URL" connections show
```

The application profile chooses table output by default.
`--json` returns the original result as canonical JSON with exact number tokens; object member order and source whitespace are not preserved.
`--events` returns runtime events, including the read observation and selected presentation.
`cli run --json` retains its original meaning of full runtime events; application defaults do not change that command.

## Embed the same documents

A consumer's executable consists of this entry point, using its own definition and profile files:
```rust
fn main() {
    std::process::exit(programmable_cli::cli_application::launch_configured_cli(
        include_bytes!("../spec/definition.json"),
        include_bytes!("../spec/application.json"),
        std::env::args_os().skip(1),
    ));
}
```

The executable name belongs to the consumer's Cargo manifest.
The prompt and domain help derive from the definition ID and current context.
No runtime specification path or `invoke` prefix is required.

## Application profile

The profile is a strict `cli-application-v1` document with these required fields:

| Field | Meaning |
|---|---|
| `format` | Exactly `cli-application-v1`. |
| `default_output` | `json` or `table`. |
| `control_aliases` | Up to 32 aliases mapping to canonical colon controls; collisions with any context or command are rejected. |
| `context_help` | Print local help after successful navigation. |
| `http.option` | Consumer-owned endpoint option such as `--url`; it cannot shadow a built-in option. |
| `http.environment` | Environment variable used when the endpoint option is absent. |
| `http.resources` | Exact set of HTTP capability IDs from the definition, at most 32; each has a fixed `path` and root-scoped `require` expression. |
| `exit_codes` | Optional mappings expressed as a required object, up to 64 error-code keys with nonzero statuses from 1 through 125. An empty object uses kernel defaults. |

Requirements use the [output expression vocabulary](../output-views/FORMAT.md#expressions), with only root available at the outer scope.
They run before a read is accepted, including under `--json`.
They validate exactly what the author declares; they do not imply full JSON Schema validation of every response member.
Paths cannot contain traversal, percent escapes, queries, fragments, credentials, or a host.
HTTP authority permits only literal `127.0.0.1` or `::1`, explicit HTTP, and a port from 1 through 65535.
The native client disables proxy discovery and redirects, uses a two-second connection deadline and seven-second total deadline, and bounds JSON bodies to one MiB.
Other hosts, TLS, authentication, and mutations need separate provider contracts.

## Options and continuation

Global options may appear before or after domain words.
`--` ends launcher parsing and passes subsequent arguments literally.
Output modes `--json`, `--table`, and `--view NAME` are mutually exclusive.
`--events` can accompany any output mode.
`--session FILE` persists navigation and observations; a new launch must grant endpoint access again.

Canonical controls remain available, including `:status`, `:views`, `:render VIEW`, and `:export FILE`.
The profile may add `?`, `up`, `top`, `help`, `tree`, and `exit` without renaming domain commands.
Definitions and profile data travel together when transferring the application surface.
Interface exports retain definitions and historical results, while process grants and presentation preferences remain outside checkpoints.
Re-rendering an exported result requires no endpoint access and performs no HTTP request.

The [contract](CONTRACT.md) defines acceptance.
AGP's `docs/rustcli/USE.md` covers its build and local live-suite workflow in that checkout.
