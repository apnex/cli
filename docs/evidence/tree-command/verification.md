# Standalone tree command

The owner asked whether `tree` should be the verb for the hierarchy.
The initial implementation reused discovery for the prototype.
This change makes the requested view explicit and keeps contextual inspection separate.

## Current behavior

- `tree` is a declared read-only composition operation with no arguments.
- Human output contains only the hierarchy for that operation.
- Machine output contains `verb_tree_text` and the active interface identity in the normal response envelope.
- `discover` returns contextual command contracts, relationships, mock state, and the last outcome.
- Help and dispatch use the same [operation declaration](../../composition/operations.json).

The existing [tree renderer](../../../src/cli_verb_tree.rs) is reused by a dedicated [composition handler](../../../src/cli_composition.rs).
No renderer or command vocabulary is embedded in the mock definition.
The [current workflow](../../composition/examples/platform/README.md) demonstrates both commands.

## Measured checks

The updated regression first failed because `tree` was unknown: [before log](before.log), exit 101.
After implementation, the same [regression](../../../tests/cli_verb_tree.rs) passed: [after log](after.log), exit 0.
It checks missing activation, clean tree output, separate discovery, immutable read requests, nested contexts, changed signatures, failed activation, draft isolation, and machine/terminal parity.

The full application suites passed 41 tests with all features and 35 in the normal build, both with exit 0.
The [instrumented log](all-features-tests.log), [normal log](normal-tests.log), [format check](format.log), and [normal build](build.log) are retained.
Formatting and the normal build returned exit 0.
The [documentation check output](scaffold-tests.log) records the final local link, layout, board, and scaffold checks separately.

Commands ran from the repository root with the [existing recorded environment](../verb-tree/toolchain-env.sh):
```sh
cargo test --locked --test cli_verb_tree -- --nocapture
cargo fmt --all -- --check
cargo test --locked --all-features --all-targets -- --nocapture
cargo test --locked --all-targets -- --nocapture
cargo build --locked --bin cli
CARGO_TARGET_DIR="$PWD/target" cargo test --locked --manifest-path tools/scaffold/Cargo.toml -- --nocapture
```

## Preserved and refreshed demo

Adding the declared verb changes the kernel declaration digest.
The existing exact-declaration checkpoint check remains enforced.
The [previous checkpoint](previous-session.json), [declaration](previous-operations.json), and [binary fingerprint](previous-binary.sha256) are retained.
The prior binary remains at `target/debug/cli-before-tree-command` for reopening previous sessions.

Before rebuilding, the prior binary [exported the active interface](previous-export.jsonl) into [the transfer bundle](platform-interface.json).
The current [construction script](workflow-construct.sh) was extracted verbatim from the guide and executed, with [construction events](construction.jsonl) retained.
The updated [CLI command sequence](workflow-use.commands) ran through the [documented opening command](workflow-open.sh), producing [human-readable output](human-output.txt).
This was a plain stdin/stdout trial, not a new interactive terminal trial.
The [tree extraction command](workflow-tree.sh) produced [tree text](verb-tree.txt) matching the fixed expected tree.

The earlier interface was then [imported](import.jsonl) into the new demo.
The [comparison](preserved-demo-check.log) confirms equality of the previous and refreshed active interface, candidate, accepted document, authoring context, and original task.
The new session has its own identity and revision sequence; [its checkpoint](session.json) ends at revision 216.
The [demo directory](demo-directory.txt) identifies the current working copy.

The [tree, discovery, and help responses](commands.jsonl) all succeeded without mutation.
The [checkpoint comparison](read-only-check.log) confirms the three reads left bytes unchanged.
The [discovery check](discovery-check.log) confirms it retains contextual metadata without a tree field.
The [refreshed tree](refreshed-tree.txt) matches the expected tree.
An initial streaming extraction helper returned exit 4 after filtering later response records; extracting from the complete response array returned exit 0.
This helper issue did not change the successful application responses.

The [source snapshot](source.sha256) and [binary fingerprint](binary.sha256) identify the checked implementation.
These are local executor observations; no independent review, remote CI, merge, or connected effects are claimed.
