# Operator configuration results

CLI-015 is complete for the [selected operator contract](CONTRACT.md).
The shared runtime provides compact discovery and process-local management selection with explicit persistence.
AGP supplies the command names and resource descriptions through its authored application profile.

## Measured acceptance

| Predicate | Observation |
|---|---|
| First-use discovery | Root help decreases from 55 to 21 lines; `ls`, `show`, and `/` work at root. Context entry gives a short command hint. |
| Complete discovery | `help --all`, `tree --all`, and structured help retain declared commands, provider details, and compatibility aliases. Mock and unbound commands retain their labels. |
| In-shell setup | `management set <url>` immediately enables the selected loopback reads. `show`, `clear`, `save`, and `load` expose and manage current and saved selections. |
| Persistence | A saved endpoint works in a new process without a launch URL. Explicit URL overrides environment, which overrides the saved default. One-shot set and clear require `--save`. |
| Rejected changes | Invalid endpoints and saved documents preserve the current selection. Stale saves preserve another session's published settings. Application mismatches, oversized files, and symlinks are rejected. |
| Portable state | Tests export and reopen observations without transporting endpoint selection or grants; historical rendering remains labeled. |
| Configured behavior | Alias equivalence and routing collisions are checked before launch. Completion exposes the same management actions and navigation words. Legacy profiles retain their existing interface. |
| AGP parity | Six native tests and twelve existing e2e tests pass, covering all thirty configured resource commands, raw responses, and independently specified table cells. |
| Installed use | The executable on the user's PATH passes a real terminal journey, saved-state reopening, and a switch from the example hub to `leaf.alpha`, with both health reads reporting ready. |

The [installed journey](../evidence/operator/installed-journey/measurements.json) records nine direct invocations and fifteen terminal steps.
The [terminal capture](../evidence/operator/installed-journey/operator-terminal.raw) includes an intentionally unconfigured read, actionable recovery, in-shell selection, save, and a live connection table.
Its final process status is 2 because that deliberate failed read remains part of the session outcome; recovery does not erase it.
Clearing the saved endpoint is followed by a fresh rejected read.
The probe uses a separate settings file, verifies the [user default remains absent](../evidence/operator/user-default-preserved.json), and stops its exact child example cleanly.

Both AGP documents still reproduce from contextual authoring commands: 4143 definition events and 1870 application events, byte-for-byte equal to the embedded documents.
The [native gate](../evidence/operator/agp-native-gate.txt) retains those measurements and the live parity assertions.

## Verification

The [check ledger](../evidence/operator/checks.tsv) retains successful gates and the original scaffold-record failure.
The [normal suite](../evidence/operator/tests-normal-final.txt) passes 100 tests; the [full instrumented suite](../evidence/operator/tests-instrumented.txt) passes 117; the [scaffold suite](../evidence/operator/tests-scaffold-corrected.txt) passes 13.
Strict Clippy passes for both application feature configurations and the scaffold package, along with formatting, scaffold drift, and the normal release build.

After the full instrumented suite compiled, one final rendering correction escaped control characters in the displayed settings filename and added its assertion.
The final normal suite, [instrumented application suite](../evidence/operator/application-instrumented-final.txt), strict lint, native gate, and release journeys cover that final source.
The [source manifest and binary measurements](../evidence/operator/measurements.json) identify exactly what was checked and installed.

## Corrections retained

The first new live AGP fixture omitted the listener required by its node configuration.
The [initial failure](../evidence/operator/agp-tests-initial.txt) and [corrected live journey](../evidence/operator/management-journey-corrected.txt) remain recorded.
The first release probe expected the example profile name `alpha` as a node ID; the example configuration and response identify it as `leaf.alpha`.
The [failed probe](../evidence/operator/release-journey.txt) remains alongside the [corrected release journey](../evidence/operator/release-journey-corrected.txt).
The scaffold checker rejected `Open;` as a backlog state token; the record was corrected to its existing state vocabulary without changing the checker.

## Scope

These results establish local management access and operator presentation for the first consumer.
Selecting an endpoint makes no request and does not certify reachability; the later resource read supplies that observation.
The new settings mechanism reuses the existing storage publisher; its crash fault scenarios remain part of the existing instrumented suite, not a separate settings-specific fault-injection study.
General remote endpoints, authentication, server configuration changes, dynamic entity contexts, and additional consumers remain outside this increment.
No hosted CI execution, independent review, production deployment, or successful IPv6 transport on this host is claimed.
