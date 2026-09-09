# Prepare and rehearse a continuation

Run from the repository root with the qualified Rust toolchain and local filesystem described in the authoring guide.
This workflow constructs a portable unfinished worker-platform session, rehearses a known completion, and checks it against the fixed task.
The rehearsal is scripted; the fresh-actor trial remains a separate step.

## Build the tools

In this workspace, load the retained toolchain environment and build the application and Rust trial tooling:
```sh
. docs/evidence/verb-tree/toolchain-env.sh
cargo build --locked --release --features continuation-trials --bin cli --bin cli-trial-receiver --example continuation-trial
```

The receiver records commands and responses without embedding the producer's completion recipe or oracle.
Ordinary application builds do not require the trial receiver feature.

## Prepare a rehearsal

Create a new trial directory and author its inputs through the CLI:
```sh
trial_root=$(mktemp -d "$PWD/target/continuation-XXXXXX")
target/release/examples/continuation-trial prepare "$PWD/target/release/cli" "$PWD/target/release/cli-trial-receiver" "$trial_root/rehearsal"
```

The `producer` directory retains the complete construction trace and `preparation.json` records costs.
Only the `package` directory is the recipient handover.
Its `manifest.json` inventories the immutable inputs, platform, identities, policy, and revisions.
The package contains a copied executable; byte sizes are reported separately from document sizes.

## Rehearse and evaluate

Run the shipped compatibility probes, documented inspection, and literal completion recipe:
```sh
target/release/examples/continuation-trial rehearse "$trial_root/rehearsal/package"
target/release/examples/continuation-trial check "$trial_root/rehearsal/package" "$trial_root/rehearsal-evaluation"
```

The evaluator checks the full definition against both tasks and exercises behavior in a separate checkpoint copy.
It checks exact values, rejected operations, schema continuity, simulated labels, and both exports.
Its result leaves the submitted checkpoint unchanged.
Rehearsal traces include the deliberately rejected stale request, incompatible declaration, and invalid commit; these failures are retained in the cost report.

## Prepare the actual handover

Prepare a separate untouched package after the rehearsal passes:
```sh
target/release/examples/continuation-trial prepare "$PWD/target/release/cli" "$PWD/target/release/cli-trial-receiver" "$trial_root/recipient"
```

Give a new actor only `recipient/package`, starting at its `START.md`.
Withhold this repository, conversation, producer traces, completion recipe, expected JSON, and evaluation tooling.
The actor must record any extra context accessed; a shared filesystem alone does not enforce that isolation.
Do not run the scripted completion against this package before the actor receives it.

The package's recorder creates `work.session.json` only on first use and retains later edits.
It never refreshes the abandoned request or overwrites a prior continuation.
After the actor finishes, the same `check` operation accepts its package and a new evaluation directory.
Actor independence, intervention, duration, and token telemetry require separate observations; a successful evaluator result does not manufacture them.

## Mechanics, rationale, and consequence

### Mechanics

Preserve immutable inputs, record CLI interaction, and compare submitted work against requirements fixed before construction.

### Rationale

The same prepared task can be rehearsed, transferred, and independently evaluated without requiring manual JSON edits.

### Consequence of violation

Reusing the rehearsed output as a supposed fresh continuation or omitting setup costs would overstate the workflow evidence.
