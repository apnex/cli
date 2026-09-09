# Mock CLI verb-tree verification

**Superseded presentation:** the owner subsequently selected a standalone `tree` verb.
The [current command record](../tree-command/verification.md) describes that change; the original measurements below are retained.

The user requested: "Can you mock up a CLI and have it print the verb structure as a tree output?"
The [platform workflow](../../composition/examples/platform/README.md) is the runnable result.
These are measured local executor observations.

## Implemented behavior

The existing read-only discovery operation now derives `verb_tree_text` from the activated definition.
The [Rust renderer](../../../src/cli_verb_tree.rs) follows context parents, lists commands before child contexts in key order, preserves required parameter order and types, and labels simulated and unbound bindings.
The [human presentation](../../../src/authoring_frontend.rs) prints the tree before the remaining contextual metadata.
The machine response retains that metadata and includes the same tree text.
Operation declarations and checkpoint formats did not change.

This keeps the displayed surface tied to the definition used for invocation.
Draft edits, failed activation, and unrelated context links cannot silently redraw the active ownership tree.

## Executed construction and output

The [command trace](../../composition/examples/platform/platform.commands) constructs and activates the platform mock from an empty document.
It supplies no serialized container literal and imports no definition.
The original bootstrap remains supplied data.

The documented construction produced 211 events, zero error events, and an activated platform definition at revision 209.
The [events](construction.jsonl), [generated definition](platform-definition.json), and [observations](construction-observations.json) are retained.
The documented exploration then inspected the simulated service, changed its replicas from 2 to 4, inspected it again, and listed releases.
The [human-readable output](human-output.txt) and [checkpoint](session.json) retain the run, ending at revision 215 in the releases context.

The [printed tree](printed-verb-tree.txt) matches the separately authored [expected tree](../../composition/examples/platform/expected-verb-tree.txt) byte for byte.
The [structured tree](structured-verb-tree.txt) also matches it byte for byte.
The [checkpoint comparison](discovery-checkpoint-check.log) confirms discovery did not change the checkpoint.
The current demonstration directory is recorded in [demo-directory.txt](demo-directory.txt); the evidence copies remain available separately.

The workflow blocks were extracted verbatim into [construction](workflow-construct.sh), [session opening](workflow-open.sh), [CLI commands](workflow-use.commands), and [tree extraction](workflow-tree.sh).
The exploration used the documented command sequence through ordinary stdin with human-readable output; no new interactive terminal trial is claimed.
The construction, exploration, tree extraction, and byte comparisons returned exit code 0.

## Local checks

| Check | Observed result | Evidence |
|---|---|---|
| New regression before implementation | Failed because discovery had no tree; exit 101. | [Failure log](tree-before.log). |
| New regression after implementation | One test passed; exit 0. | [Focused log](tree-after.log), [assertions](../../../tests/cli_verb_tree.rs). |
| All application targets with all features | 41 tests passed; exit 0. | [Full log](all-features-tests.log). |
| All application targets in the normal build | 35 tests passed; exit 0. | [Normal log](normal-tests.log). |
| Application formatting | Passed; exit 0. | [Formatting log](format.log). |
| Documentation and scaffold checks | 13 tests passed; exit 0. | [Scaffold log](scaffold-tests.log). |
| Generated layer views | Seven layers and 15 views synchronized; exit 0. | [View check](scaffold-views.log). |

The regression checks nested and empty contexts, a command sharing a child's name, parameter type changes, related-context cycles, draft isolation, failed activation, read-only checkpoint bytes, and machine/terminal result parity after reopen.
The previous application regressions remain in both full runs.
The [regression artifacts](regressions/) retain this run separately from the original composition closeout.

Commands ran from the repository root with the [recorded environment](toolchain-env.sh):
```sh
cargo test --locked --test cli_verb_tree -- --nocapture
cargo test --locked --all-features --all-targets -- --nocapture
cargo fmt --all -- --check
cargo test --locked --all-targets -- --nocapture
cargo build --locked --bin cli
CARGO_TARGET_DIR="$PWD/target" cargo test --locked --manifest-path tools/scaffold/Cargo.toml -- --nocapture
CARGO_TARGET_DIR="$PWD/target" cargo run --locked --manifest-path tools/scaffold/Cargo.toml -- --check
```

The [source fingerprint](source.sha256) covers application source, tests, declarations, and the example inputs.
The [source comparison](source-check.log) and [binary comparison](binary-check.log) passed against that snapshot and the [normal binary fingerprint](binary.sha256).

The Rust toolchain was restored in a persistent user-local directory without changing shell profiles.
Its [installation output](toolchain-install.log) and [measured versions](toolchain-version.log) are retained.
The original composition closeout's missing temporary-toolchain observation remains historical evidence.

This is local implementation and demonstration evidence, with no claim of an independent agent trial, merged change, remote CI, or real platform effects.
