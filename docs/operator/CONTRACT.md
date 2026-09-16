# Contextual operator experience

The [owner selection](../context/cli-015-selection.json) authorizes CLI-015.
The observed first-use session repeatedly prints endpoint and provider metadata, rejects root listing and slash navigation, and cannot configure its endpoint from inside the shell.

## Selected behavior

An optional application operator profile selects compact presentation, declared command aliases, and context-listing words.
Aliases must reference equivalent declared commands; human summaries may collapse them while structured discovery remains complete.
Domain commands take precedence over listing fallbacks.
The existing explicit run interface and profiles without operator settings retain their behavior.

AGP supplies `management` as the alias of the shared `:endpoint` control.
`management show`, `management set <url>`, `management clear`, `management save`, and `management load` work at any context and in direct invocations.
Set and clear take effect in the current process; save explicitly publishes the selection for future launches.
One-shot set and clear require `--save`, so an acknowledged change survives the process that performed it.
The application uses `--config FILE` when supplied, otherwise the user's configuration directory under `programmable-cli/<application-id>/management.json`.
Endpoint precedence is explicit URL, environment, then saved selection.
No network request occurs merely because an endpoint is selected.
Existing loopback restrictions and request bounds continue to apply.

Settings use a separate bounded document, application identity, the existing atomic storage publisher, and observed-byte conflict detection.
A rejected setting or failed save must preserve the current endpoint or previously saved bytes respectively; publication uncertainty remains explicit.
Portable interface exports and run checkpoints contain no endpoint selection or authorization.
Re-rendering retained output remains explicitly historical after changing endpoints.

Root `ls` and `show` list available contexts, while configured resource commands retain their existing behavior.
Slash returns to root through an explicitly configured alias.
Context entry prints a short hint; explicit help provides local commands and details.
Detailed help and structured events retain full definitions and machine-readable error codes.

## Acceptance

Reproduce the owner's fresh-shell discovery and recovery sequence.
Configure one live endpoint inside the shell, switch to another, reject an invalid replacement without losing the first selection, clear, save, and reopen.
Prove saved-setting precedence, application isolation, concurrent-save rejection, and absence of endpoint authority in exports.
Require completion and help to describe commands that actually execute.
Retain all AGP management data and existing table parity checks.
Install the validated updated executable in the user's existing local binary directory.

## Design audit

Verdict: pass-with-guardrails before implementation, engineering self-audit under mission-kit M7.
Inputs are the owner's transcript, shared run routing/help, application launcher, HTTP grants, storage publication, and native consumer tests.

| Anchor | Implementation guardrail |
|---|---|
| A1 and A5: visible state | Distinguish selected endpoint, saved default, and historical data; selection never implies observed reachability. |
| A2: configured behavior | Alias declarations, presentation, and completion share validated configuration. |
| A3: composition | Shared CLI owns mechanisms; AGP owns management vocabulary and resource descriptions. The owner selected shared mechanisms for the first consumer; multi-consumer recurrence is unmeasured. |
| A4: fidelity | Keep complete structured discovery, existing data tables, and prior evidence. |
| A7 and A8: recoverable integrity | Validate before replacing grants; use atomic storage and refuse stale saves. |
| A13: selected intent | Continue the approved operator improvements without requesting authorization again. |

The earlier launch-only authority boundary is extended by explicit operator selection and a separately saved local preference.
That preference is an application input and never travels inside portable interface state.
Closeout must measure these guardrails; this audit does not claim an independent reviewer or deployment verdict.
