# Contribute

Read the [vision](../VISION.md), [board](BOARD.md), and the contract for the workflow you intend to change.\
The project owner selects product direction; the existing selected scope authorizes its ordinary implementation and verification work.\
The [workspace instructions](../AGENTS.md) govern engineering claims and evidence.

## Build environment

Use the [getting-started prerequisites](GETTING-STARTED.md#prerequisites).\
The pinned toolchain is declared in [rust-toolchain.toml](../rust-toolchain.toml), and dependency resolution is retained in both Cargo lockfiles.\
The Linux terminal test also requires `script` from util-linux.\
Shell walkthroughs use `jq`; individual advanced guides name additional commands such as `rg`.

Build the production executable from the repository root:
```sh
cargo build --locked --release --bin cli
```

Application code and repository tooling are Rust.\
The [scaffold package](../tools/scaffold/Cargo.toml) is a separate package with its own lockfile.\
Its build output can share the root `target` directory through `CARGO_TARGET_DIR`.

---

## Source map

| Concern | Start reading |
|---|---|
| Launcher and authoring presentation | [main](../src/main.rs), [authoring frontend](../src/authoring_frontend.rs) |
| Exact document values and locations | [document values](../src/document_value.rs), [document paths](../src/document_path.rs) |
| Session transactions and publication | [authoring runtime](../src/authoring_runtime.rs), [session storage](../src/session_storage.rs) |
| Declared operation contracts and handler checks | [operation definitions](../src/operation_definition.rs), [authoring operations](../src/authoring_operations.rs) |
| Schema interpretation and contextual guidance | [constraints](../src/schema_constraint.rs), [guidance](../src/schema_guidance.rs) |
| Configured CLI definitions and active state | [definitions](../src/cli_definition.rs), [active interface](../src/cli_interface.rs) |
| Reusable component composition | [assembly](../src/cli_assembly.rs) |
| Granted external observations | [JSON file reads](../src/cli_file_read.rs) |
| Direct commands, help, and completion | [run routes](../src/cli_run_routes.rs), [run frontend](../src/cli_run_frontend.rs) |
| Acceptance tests and fault scenarios | [tests](../tests/), [test fault controls](../src/storage_faults.rs) |
| Documentation declarations and rendering | [layer registry](layers.json), [scaffold renderer](../tools/scaffold/src/main.rs) |

These are internal Rust boundaries, not separately versioned public libraries.

---

## Verification

Run these commands sequentially from the repository root:
```sh
cargo test --locked --all-targets
cargo test --locked --all-features --all-targets
cargo fmt --all -- --check
CARGO_TARGET_DIR="$PWD/target" cargo test --locked --manifest-path tools/scaffold/Cargo.toml
CARGO_TARGET_DIR="$PWD/target" cargo run --locked --manifest-path tools/scaffold/Cargo.toml -- --check
cargo fmt --manifest-path tools/scaffold/Cargo.toml -- --check
cargo build --locked --release --bin cli
```

The normal and instrumented suites exercise different feature configurations.\
The instrumented suite includes full continuation experiments and can take several minutes.\
The final build produces the normal executable after tests; distribution must not enable `fault-injection`, `contract-probes`, or `continuation-trials`.\
Application tests execute real requests, including rejected changes and persistence failures.\
Scaffold tests check document placement, links, declaration views, and board/backlog consistency; they do not prove application behavior.

Execute any changed user workflow literally against the built binary.\
Capture output before filtering it and record the command's own exit code.\
If a command needs an undocumented step, fix the document and rerun the affected journey.\
When retaining application fixtures, set `CLI_ACCEPTANCE_EVIDENCE_DIR` to a fresh directory under `target` for local work or under `docs/evidence` for a deliberately retained record.

Clippy is an additional diagnostic, not an established clean gate for this prototype.\
The [publication review](publication/REVIEW.md) records the initial strict-lint result and its disposition.\
Do not silence diagnostics or restructure the error protocol merely to make a publishing sweep look clean.

---

## Change declarations and documentation

The [authoring](authoring/operations.json), [composition](composition/operations.json), and [constraint](constraints/operations.json) declarations are runtime inputs.\
Changing them changes the kernel identity that saved checkpoints require.\
Retain the original binary/declaration pair for old checkpoints, or transfer a portable interface export under the documented contract.

The layer registry owns the marked region in the architecture and the local responsibility views.\
Regenerate those views after a registry change:
```sh
CARGO_TARGET_DIR="$PWD/target" cargo run --locked --manifest-path tools/scaffold/Cargo.toml -- --write
```

Keep documentation under `docs/`, except root `README.md` and `VISION.md`.\
Use plain ASCII Markdown, semantic sentence breaks, runnable command blocks, and links to the authoritative contract.\
Preserve completed tasks, approvals, original oracles, failures, and decision records; correct a current guide without rewriting its historical evidence.\
Update board and backlog together when work changes state.

---

## Propose a change

A useful issue or patch states the concrete task, observed behavior, expected behavior, and a reproduction with sensitive data removed.\
A behavior change needs evidence against the original task, including relevant failure and recovery cases.\
Do not infer universal coverage or workflow savings from a new demonstration.

Keep each change concerned with one problem and cite the checks that actually ran.\
A commit message describes that problem without AI or tool attribution.\
The project uses the owner-selected [MIT license](../LICENSE).\
Release-channel decisions remain with the owner under the [publication guide](PUBLISHING.md).

---

## Mechanics, rationale, and consequence

### Mechanics

Read the owning contract, change its implementation or declaration, and run the checks and literal user journeys affected by that change.

### Rationale

The same declaration must continue to describe what humans and agents can discover and execute.

### Consequence of violation

An apparently local improvement can strand checkpoints, drift generated views, or make a simulated result appear connected.
