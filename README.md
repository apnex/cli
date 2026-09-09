# Programmable CLI

Construct JSON and reusable CLI specifications through contextual commands, then run those specifications as ordinary CLIs.

**Status: experimental.**\
The current implementation supports local Linux workflows; definition formats and Rust interfaces are not stable public APIs.

## Install

From a source checkout with Rust, Cargo, and a C linker available:
```sh
cargo build --locked --release --bin cli
./target/release/cli --help
```

See [getting started](docs/GETTING-STARTED.md#prerequisites) for filesystem requirements and the tested environment.

---

## Use

Explore the included service-catalog specification and call one of its configured verbs:
```sh
./target/release/cli run docs/connected/acceptance/expected-definition.json :tree
./target/release/cli run docs/connected/acceptance/expected-definition.json services quota 12
```

The quota command returns `{"quota":12}` on stdout and labels its simulated result on stderr.\
[Getting started](docs/GETTING-STARTED.md) takes you from contextual JSON authoring to constructing, exporting, and running your own CLI without manually editing JSON.\
The [workflow index](docs/README.md#choose-a-workflow) covers schemas, reusable components, connected reads, and agent continuation.

---

## Test

Run the normal application suite:
```sh
cargo test --locked --all-targets
```

The [contributor guide](docs/CONTRIBUTING.md#verification) includes the required fault-instrumented suite, formatting, and documentation checks; the Linux terminal test requires `script` from util-linux.\
The [publication review](docs/publication/REVIEW.md) records what was actually checked and the remaining limits.

---

## Remove

Remove local build products:
```sh
cargo clean
```

Your checkpoints and exports remain wherever you saved them.\
See [removal and retained data](docs/GETTING-STARTED.md#remove) for an optional installed binary and session files.

---

## Project

The [VISION](VISION.md) defines the purpose: contextual construction, reusable definitions, deterministic interpretation, and work another actor can resume.\
The ambition is a kernel that can be configured or assembled into any CLI a project needs; the current capability set is bounded.

| Document | What it answers |
|---|---|
| [Architecture](docs/ARCHITECTURE.md) | What responsibilities compose the system, which contracts exist, and which boundaries remain provisional? |
| [Board](docs/BOARD.md) | What is complete, selected, or held, and what is the next useful move? |
| [Documentation index](docs/README.md) | Where are the user guides, contracts, and historical evidence? |
| [Contributing](docs/CONTRIBUTING.md) | How do I build, change, and verify the project? |
| [Publication](docs/PUBLISHING.md) | What is prepared, and what must be decided before a release? |

All project documentation is under `docs/`, except this README and the root VISION.\
`AGENTS.md` is the governing workspace instruction file.\
The project uses the [MIT license](LICENSE); third-party material retains its own license.
