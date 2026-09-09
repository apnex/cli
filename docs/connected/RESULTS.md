# Connected JSON read results

**CLI-006 is complete within the selected local JSON read scope.**
The [contract](CONTRACT.md), [task](acceptance/TASK.md), and pre-implementation [audit](AXIOM-AUDIT.md) define this claim.
The [verification record](../evidence/connected-json-read/verification.json) retains commands, results, source identity, and limits.
This is executor-run local evidence, not independent assurance, a deployment, or an agent performance trial.

## Implemented behavior

A definition can bind service inspection to the native `json-file-read-v1` provider through a logical capability.
The receiving process grants a file with `--grant-json-read services.catalog catalog.json`.
One Rust provider opens that filename relative to a pinned directory, rejects symbolic links and special files, and reads bounded JSON with exact number tokens.
The binding interpreter, discovery, tree, activation, export, and checkpoint validation share the same declared meaning.
Grants are not serialized; importing a definition or historical observation cannot recreate them.

The [literal walkthrough](USE.md) authored and exported the CLI, imported it in another session, read the initial catalog, observed changed source bytes, and compared the unchanged exported specification.
A third session imported the historical interface without a grant, could inspect its old observation, and rejected a fresh invocation.
It reopened without the imported interface file, received its own grant, and observed the later catalog.
The run remains at the path in [walkthrough location](../evidence/connected-json-read/walkthrough-location.txt); its files are also retained under [walkthrough evidence](../evidence/connected-json-read/walkthrough/author/author.jsonl).

The generated tree is:

```text
service-catalog-connected
`-- services/
    |-- invoke inspect [connected:json-file-read-v1]
    |-- invoke quota <value:number> [simulated]
    `-- invoke restart [unbound]
```

Inspection observes a file; it does not establish service liveness.
Quota still changes only simulated state, and restart remains unbound.

## Measured validation

| Check | Observed result |
|---|---|
| Complete instrumented application suite | 77 passed, 0 failed |
| Complete normal-build application suite | 61 passed, 0 failed |
| New connected acceptance tests | 8 passed with instrumentation; 7 in the normal build |
| Literal walkthrough | All blocks passed using the normal debug binary; the expected missing-grant rejection was checked explicitly |
| Documentation and formatting | 13 tests passed; 7 layers and 15 generated views agree; Rust formatting passed |

The [new application tests](../../tests/connected_json_read.rs) assert the effect, not just handler execution:

- Exact bytes and digests distinguish successive external observations; complete checkpoint comparison excludes changes to authoring state, constraints, and mock state.
- Terminal and machine consumers agree, including payloads that imitate outcome labels.
- Exact retries retain historical output after source replacement, deletion, and grant-free reopen.
- Plain and assembled interface transfer preserve connected requirements without transferring authority; independently scoped mock state remains isolated.
- Renaming a granted parent directory and substituting a symlink does not redirect reads.
- Bad targets, malformed or oversized data, extra arguments, stale revisions, and invalid launcher grants reject without checkpoint mutation.
- Applied definition, outcome, and checkpoint mutations reject; the test output records that each changed input landed before evaluation.
- Four confirmed fault markers exercise publication failure and uncertainty in both presentations. Before replacement, the old checkpoint survives and a later retry can observe new bytes. After replacement, source-free recovery replays the published observation.

The [application log](../evidence/connected-json-read/application-tests.log) and [normal-build log](../evidence/connected-json-read/normal-tests.log) retain the complete regressions, including earlier authoring, constraint, composition, import, and continuation checks.
Historical CLI-005 comparison measurements remain historical; this increment makes no new workflow-economics claim.

## Retained failures and corrections

Before implementation, the [fixed acceptance test failed](../evidence/connected-json-read/failing-before.log) because activation rejected the unsupported connected binding.
Construction and independent expected-document comparison had already succeeded.
This is the behavioral before/after evidence.

Two ordinary compile corrections are retained separately: a recovery message needed a borrowed string, and a test compared an optional result with an unwrapped value.
The [first compile log](../evidence/connected-json-read/compile.log), [repeated compile failure](../evidence/connected-json-read/first-supported.log), and [test compile log](../evidence/connected-json-read/focused-first.log) are not behavioral failures or passes.
After those repairs, the [complete focused run](../evidence/connected-json-read/focused.log) passed all eight tests without changing the task oracle.

## Limits and continuation

The concrete provider is a local JSON file read, with no configurable executable, subprocess, network request, or external content write.
The read and checkpoint publication are not one external transaction: a failed publication can follow a completed read.
Concurrent in-place source writes may yield the bytes read across that interval; no atomic source-snapshot or filesystem deadline guarantee is claimed.
Source digests record observed bytes; recovery verifies internal consistency and does not authenticate a modified checkpoint or remeasure historical source bytes.
Reads can update filesystem metadata such as access time.

The native implementation and tests were run on local Linux with Rust 1.98.1.
The new direct `rustix` dependency uses the already locked version 1.1.4; the lockfile adds only the root package's dependency edge.
The [previous runtime pair](../evidence/connected-json-read/before/runtime-location.txt) and its [hashes](../evidence/connected-json-read/before/runtime.sha256) are preserved.
Updated composition declaration bytes require their matching binary for checkpoint reopen; no silent migration is implemented.

A concrete mutating or remote operation must establish its authority, protocol, failure, and retry requirements before extending the provider boundary.
CLI-007 separately awaits a representative API task for OpenAPI-derived invocation.
A reproducible violation of the supported read, authority, fidelity, transfer, or recovery contract reopens CLI-006.

## Mechanics, rationale, and consequence

### Mechanics

Connect a declared operation through an explicitly granted native provider and verify real observations, transfer, and recovery through actual CLI processes.

### Rationale

This establishes a usable first connected boundary without promoting simulation or persisted history into evidence of a fresh read.

### Consequence of violation

A future actor could otherwise assume that receiving a CLI definition also grants access or that a saved result describes current external state.
