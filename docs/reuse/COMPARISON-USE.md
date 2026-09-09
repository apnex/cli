# Run the controlled authoring comparison

Run from the repository root using the local toolchain from the [authoring guide](../authoring/USE.md).
Read the [comparison contract](COMPARISON-CONTRACT.md) for the two acceptance verdicts and timing limits.
The commands use four known scripted methods; the prior fresh-agent observation remains separate.
The [historical comparison results](COMPARISON.md) retain the three-method experiment before import existed.
The current tool adds `targeted-json-import`, using the same literal edits followed by candidate import, commit, and activation.

## Build and execute

Build the Rust application and comparison tool:

```sh
. docs/evidence/verb-tree/toolchain-env.sh
cargo build --locked --release --features continuation-trials --bin cli --bin cli-trial-receiver --example continuation-trial
```

Prepare shared inputs and run each method in a new directory:

```sh
comparison_root=$(mktemp -d "$PWD/target/authoring-comparison-XXXXXX")
target/release/examples/continuation-trial compare "$PWD/target/release/cli" "$PWD/target/release/cli-trial-receiver" "$comparison_root/run"
```

The tool refuses to overwrite an existing run directory.
Unexpected method failures are retained in `comparison.json` and cause a nonzero command exit after the other methods have been attempted.
The strict staged-continuation failure expected for file editing remains a separate verdict even when the artifact comparison exits successfully.

---

## Read the result

Inspect the measured phases and both verdicts:

```sh
jq '.methods[] | {method, status, full_staged_continuation: .evaluation.full_staged_continuation, phases: .workflow.phases, external_work: .workflow.external_work}' "$comparison_root/run/comparison.json"
```

| Artifact | Contents |
|---|---|
| `run/shared/preparation.json` | Full source, policy, and partial-draft construction costs. |
| `run/recipes/` | Snapshots of the exact command and text-edit recipes used by the methods. |
| `run/<method>/result.json` | Method outcome, full-continuation verdict, phase counts, external work, durations, and artifact sizes. |
| `run/<method>/package/logs/` | Actual CLI inputs, outputs, errors, and process metrics. |
| `run/targeted-json-editing/package/external-edit/` | Exact document before/after, applied edit recipe, and measured file work. |
| `run/<method>/evaluation/` | Disposable runtime probes; submitted files remain separate. |

The original `targeted-json-editing` method still validates the edited file and activates it directly, leaving staged documents unfinished.
The added `targeted-json-import` method retains the external work, imports its edited document, restores the task's authoring context, commits under the unchanged policy, and activates the candidate.
Its edited source and final canonical definition export have separate filenames, preserving both exact files.
CLI traffic alone is neither method's full input cost, and a staged-state pass does not turn external text editing into command-only authorship.
Do not add overlapping durations or compare scripted execution time to the historical fresh-agent observation window.

---

## Mechanics, rationale, and consequence

### Mechanics

Create independent copies, execute retained recipes, and inspect both acceptance verdicts alongside their scoped measurements.

### Rationale

The same final interface can be produced through different workflows whose continuation guarantees differ.

### Consequence of violation

Reading only the top-level pass or CLI input count could hide the activation-only method's staged-state failure and omit work performed outside the CLI.
