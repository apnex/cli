# First connected read contract

**Status: implemented within the selected CLI-006 scope; [results](RESULTS.md) retain the measured validation.**
The [approval record](../context/cli-006-approval.json) distinguishes user authorization from the engineering choice of reference capability.
The [acceptance task](acceptance/TASK.md) fixes observable behavior before code.

## Interpretation

The existing service inspection command can read a catalog maintained outside the CLI session.
The result is an observation of JSON bytes in a local file, never evidence that a service is running.
Its definition declares a logical capability and the native `json-file-read-v1` provider.
Its receiving process separately grants that capability access to one file.
This is the first connected provider; arbitrary executable hooks and external writes remain separate work.

A connected binding has exactly `kind: connected`, `provider: json-file-read-v1`, and `capability: <identifier>`.
The identifier uses existing CLI identifier rules.
This provider accepts no parameters and reads the complete granted JSON document.
Definitions with unsupported providers, extra fields, or parameters are rejected before activation.
Ordinary `cli-definition-v1` and assembled `cli-definition-v2` carry the new binding vocabulary; older binaries reject it explicitly.

## Authority and target

The launcher accepts repeated `--grant-json-read <capability> <file>` only with `--compose`.
Grant identifiers must be unique; at most 32 are accepted.
The native dispatcher owns transient grants alongside its registered handlers.
Authored documents, interface exports, imported observations, checkpoints, and origin records cannot create grants.
Definitions remain activatable and discoverable without grants.
Discovery distinguishes a declared connected binding from a grant available in this process.
A grant reports authority, not file availability or service health.

A grant pins the supplied parent directory when the process starts, then each fresh invocation opens the named file within that directory.
Relative targets are resolved from the launch working directory.
Replacing the file with another regular file is observed on the next invocation.
Renaming the directory and substituting a symlink at its former path cannot redirect the grant.
The final path component must be a regular file; symbolic links and special files are rejected.
Reads are bounded to 1 MiB, with existing JSON depth and scalar limits, duplicate-key rejection, and exact numeric token preservation.
No subprocess or network transport is introduced.
This Unix implementation makes no deadline guarantee for an unresponsive filesystem and is not a general process sandbox.
Opening and reading can have ordinary filesystem metadata effects such as access-time updates; file content is never written by this provider.

## Invocation, receipt, and recovery

Both presentations dispatch the same configured `invoke` operation.
A successful connected result has runtime-owned `binding: connected`, `effect: external_read`, zero simulated steps, canonical `output_json_text`, and an observation containing provider, capability, raw source byte count and SHA-256, and output SHA-256.
The output digest covers the exact canonical output text, preserving numeric spelling.
Raw source identity is recorded provenance; source-free recovery cannot remeasure historical raw bytes.
Concurrent in-place source writes may produce bytes read across that interval; no atomic source snapshot is promised.
An authored payload cannot replace these outer labels.
The invocation changes only the interface's last outcome and the usual session revision/receipt; candidate, accepted document, authoring context, schema attachment, and mock state are unchanged.

The existing runtime checkpoints the observation and receipt before acknowledging success.
An exact retry of the latest persisted request returns that historical observation without rereading a changed or missing file, even after reopening without a grant.
A new request requires a new read and current process authority.
Reopen, discovery, and interface export never refresh an observation implicitly.
Observation history and authority must remain distinguishable during transfer.

If a read or parse fails, no observation or checkpoint mutation is published.
If checkpoint publication fails before replacement, the original checkpoint survives, but a read may already have occurred.
If publication is uncertain, reopen and inspect the receipt before retrying.
The provider performs no content mutation, so repeating an unacknowledged read has no external content write to duplicate; it can observe different bytes.
This contract does not extend local transaction guarantees to future external writes.

## Errors and composition

| Condition | Required result |
|---|---|
| Missing grant | `CAPABILITY_NOT_GRANTED`, with the logical capability and grant syntax |
| Invalid, duplicate, or excessive launcher grant | `INVALID_CAPABILITY_GRANT`; no session is created |
| Missing, unreadable, symbolic-link, or special target | `CONNECTED_READ_FAILED`; checkpoint unchanged |
| Malformed JSON, duplicate keys, or invalid UTF-8 | `INVALID_CONNECTED_DOCUMENT`; checkpoint unchanged |
| Source/document/scalar/depth limit | `LIMIT_EXCEEDED`; checkpoint unchanged |
| Wrong invocation signature or stale revision | Existing request rejection before opening the target |
| Unsupported binding/provider or connected parameters | `INVALID_CLI_DEFINITION`; active interface unchanged |

Assembly namespaces command identities and simulated state as before.
Logical read capability identities are preserved, so multiple components may intentionally require the same read authority.
Assembly snapshots, validation, and source-free transfer cannot smuggle a grant or silently replace connected behavior with a mock.
Tree rendering labels the connected provider; discovery reports the required logical capability and current grant availability.
Saved outcome validation checks binding/provider/capability, zero simulated steps, output digest, source-count bounds, and all existing interface relationships without reading any file.
It checks internal consistency, not cryptographic authenticity of an untrusted checkpoint.

## Acceptance predicate

Pass requires command-only definition construction and independent expected-definition equality; separate-session import/activation/use; paired terminal and machine observations of real changes to the granted file; unchanged mock and authoring state; missing-authority and invalid-input rejection; source-free exact receipt replay; grant-free handover with historical observations; component assembly; and observed publication failures on both sides of checkpoint replacement.
Negative tests must apply changed inputs before asserting rejection.
Execute the documented workflow literally and retain source/binary identities and failed runs.
Old declaration/binary pairs are preserved before changing composition metadata; no checkpoint migration is implied.

## Mechanics, rationale, and consequence

### Mechanics

Resolve a configured binding through a transient read grant, record its observation through the existing checkpoint transaction, and project the same labels to each presentation.

### Rationale

A small real consumer earns the authority and recovery boundary before a general hook transport is selected.

### Consequence of violation

A portable definition could acquire unintended authority, or a saved observation could be mistaken for a new read or a running service.
