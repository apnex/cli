# Programmable CLI

A project for constructing reusable CLI definitions through contextual interaction, with portable meaning and resumable engineering context.

**Status: Rust JSON authoring, CLI construction and mocks, and schema constraints are implemented.**
Construct JSON, schemas, OpenAPI descriptions, and CLI definitions through contextual commands. Explicitly attach an authored schema to guide and validate another document, or activate a CLI definition with discoverable contexts and labeled mock behavior.

## Install

Build locally with Rust and Cargo:

```sh
cargo build --locked --bin cli
target/debug/cli --help
```

The application is currently verified on local Linux with Rust 1.98.1.
The [usage guide](docs/authoring/USE.md) gives prerequisites and complete session examples.

---

## Use

Use the [authoring guide](docs/authoring/USE.md) to construct, save, and resume JSON through commands such as `edit`, `set`, `show`, `commit`, and `discard`.
Use the [CLI construction guide](docs/composition/USE.md) to build a service-catalog interface entirely through authoring commands, activate it with `--compose`, and exercise its configured operations.
Use the [schema authoring guide](docs/constraints/USE.md) to construct a JSON Schema without raw container literals, attach it with `--constraints`, repair an instance, and resume its saved state.
Use the [reuse walkthrough](docs/reuse/USE.md) to prepare an unfinished worker-platform prototype and rehearse its continuation through the CLI.
The [experiment record](docs/reuse/CLI-005.md) keeps scripted verification separate from the pending fresh-actor trial.

Start with the [vision](VISION.md) for project intent, then follow the [working approach](docs/approach.md) and [target architecture proposal](docs/ARCHITECTURE.md).
The [project board](docs/BOARD.md) shows proposed next moves, dependencies, and the decisions they require.
The [continuation guide](docs/resume-work.md) identifies accepted intent, proposed structure, and unresolved choices.
The [implementation record](docs/authoring/CLI-002.md) carries the first journey's actual application evidence.
The [composition record](docs/composition/CLI-003.md) covers self-authorship, explicit activation, mock outcomes, and portable interface state.
The [constraints record](docs/constraints/CLI-004.md) covers the supported Draft 2020-12 subset, contextual guidance, constrained commits, and OpenAPI authoring as data.

Project documentation lives under `docs/`, except this `README.md` and the root `VISION.md`.
`AGENTS.md` is the workspace's governing instruction file.

---

## Test and verify

```sh
cargo test --locked --all-targets
cargo test --locked --all-features --all-targets
cargo fmt --all -- --check
```

The application suite covers authoring and construction journeys, schema interpretation, exact values, rejected changes, atomic mocks, recovery, transfer, limits, and declaration drift.
The second command enables explicit test instrumentation.
The [scaffold verification workflow](docs/scaffold-workflow.md#verify-the-scaffold) checks documentation consistency separately.

---

## Remove

Use `cargo clean` to remove local build output.
Preserve checkpoints and exports you intend to keep; the [removal guidance](docs/authoring/USE.md#verify-and-remove) explains session files and inactive locks.
No service is installed.

---

## Project records

- [Captured project intent](docs/project-intent.md)
- [Original discussion and correction provenance](docs/context/README.md)
- [Layer declaration](docs/layers.json)
- [Architecture with generated responsibility map](docs/ARCHITECTURE.md)
- [Board of proposed next moves](docs/BOARD.md)
- [Durable work record](docs/BACKLOG.md)
- [First authoring contract and acceptance design](docs/authoring/CLI-001.md)
- [Implemented authoring slice and evidence](docs/authoring/CLI-002.md)
- [Implemented CLI construction and mocks](docs/composition/CLI-003.md)
- [Implemented schema construction and constraints](docs/constraints/CLI-004.md)

The layer declaration describes proposed responsibility boundaries; implementation records identify the behavior currently available.
