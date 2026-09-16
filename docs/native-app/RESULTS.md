# Native application integration results

The shared Rust runtime now hosts AGP's complete existing HTTP management surface as authored configuration.
AGP's `rustcli` consumer embeds ten contexts, thirty domain commands, and ten native table views.
Shared mechanisms include the application launcher, HTTP read grants, control aliases, contextual help, and object-entry/count projections.
No AGP names occur in those shared mechanism modules.

## Measured acceptance

| Predicate | Observation |
|---|---|
| Complete consumer | Every management resource is checked against its public SDK response and live HTTP response; all three configured command forms execute in raw JSON and table-event modes. |
| Useful views | Each table's display cells are compared with separately written expectations; original connection/route parity helpers remain unchanged apart from explicit executable selection. |
| Nonempty maps | A controlled HTTP resource case verifies sorted gauge names and exact current/maximum/high-water values beyond JavaScript's integer precision. The live Loopback node's global gauge map is empty. |
| Contextual construction | Both documents replay from authoring commands and compare byte for byte with their embedded JSON. Construction initially emitted 4121 definition events and 1840 application-profile events. |
| Real operator use | Eight guide invocations and an actual terminal session pass against a running example hub using the installed release executable; the exact child example exits cleanly afterward. |
| Native execution | All-resource tests invoke the executable with an empty PATH. HTTP, projection, and rendering require no helper executable. |
| Continuation | Failed reads preserve the previous observation; export/reopen supports historical rendering without authority and rejects a fresh ungranted read. |
| Failures | Invalid endpoints, proxies, redirects, HTTP statuses, malformed JSON, response requirements, body limits, and presentation failures are exercised. A stalled body reaches the total deadline and exits as transport failure with empty stdout. |
| Compatibility | Frozen output oracles remain unchanged; generic run's existing JSON event semantics and legacy file-observation serialization remain accepted. |

The [source and binary measurements](../evidence/native-app/measurements.json) identify the baselines, tested code, and installed executable.
The [check ledger](../evidence/native-app/cli-checks.tsv) records 92 normal tests, 109 instrumented tests, 13 scaffold tests, all three strict lint commands, formatting, scaffold drift, release build, and the unchanged frozen oracle.
The [operator record](../evidence/native-app/initial-operator-guide.json), [terminal capture](../evidence/native-app/initial-operator-terminal.raw), and [deadline check](../evidence/native-app/initial-agp-operator-deadline.txt) retain direct process observations.
AGP's consumer record lives at `docs/rustcli/INTEGRATION.md` in its repository.
The complete [native integration gate](../evidence/native-app/agp-native-gate.txt) passes five consumer tests and the existing twelve-file e2e suite after rebuilding and replaying both documents.

## Corrections retained

The first raw-JSON test assumed source member order; the kernel contract preserves exact values and canonicalizes objects.
The assertion now checks canonical output while retaining the exact large integer.
A new advertisement expectation initially compared display text with a numeric wire revision; its independently read DTO defines that field as numeric, so the expected display is its string form.
An initial completion test used a field where Reedline exposes a method; compilation caught that error.

The initial board update used an unsupported state string in CLI and omitted the selected AGP item's build-order row.
Both record checks rejected the updates; the board entries were corrected without changing their checkers.
The earlier failing captures remain alongside the passing checks.

The initial IPv6 fixture could not bind `::1` because this host has loopback IPv6 disabled.
Successful IPv6 reads are `UNRUNNABLE` here, not claimed passing.
The [explicit boundary run](../evidence/native-app/initial-ipv6-boundary.txt) verifies the valid literal grant returns the transport error when unavailable; the same test exercises a successful read on hosts where the bind succeeds.

## Scope of proof

These observations cover local live AGP nodes, an installed Linux executable, HTTP failure fixtures, and the existing kernel contracts.
They do not establish a production deployment, successful IPv6 transport on this host, an independent implementation review, or arbitrary remote/authenticated/mutating providers.
The profile checks the response requirements its author declares, rather than asserting complete response-schema certification.
HTTP bodies retain the one-MiB client limit even when a server permits more.
CLI formats and Rust interfaces remain experimental.
