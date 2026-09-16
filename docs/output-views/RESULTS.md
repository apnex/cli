# Native AGP output-view results

**Status: implemented and locally verified for the selected AGP consumers.**
Scope: [CLI-013](../BACKLOG.md#cli-013), the [selected contract](CONTRACT.md), and the [original acceptance task](acceptance/TASK.md).
The owner selected AGP's connection and route views and required native Rust execution.
The source research remains under [command output views](../research/command-output-views/README.md).

## Delivered behavior

The Rust executable interprets named views in `cli-definition-v3`.
Views contain input requirements, typed paths and expressions, ordered columns, and display formats.
`cli run --table` selects a command's view, `--view` selects a named override, and `--json` retains the original invocation alongside presentation.
`:views` exposes the definitions, `:render` presents a retained invocation, and `cli render` previews a local document without invoking a binding.
The [consumer guide](USE.md) and [format reference](FORMAT.md) describe these surfaces.

The [529-line recipe](acceptance/agp.commands) constructs both commands and views through contextual authoring, with no serialized JSON containers.
Its [generated definition](acceptance/agp.json) binds the two AGP verbs to granted native file reads.
No AGP domain fields or command names enter the shared Rust projection/formatting implementation.
Both views transfer in standalone definitions, interface bundles, and namespaced component assemblies.

## Acceptance observations

The [Rust acceptance suite](../../tests/cli_output_views.rs) executes eleven tests with assertions against the task, frozen references, runtime state, and actual process outcomes.
The following observations are measured locally; they do not claim live AGP service behavior or independent certification.

| Task | Observation and evidence |
|---|---|
| 1. Contextual construction | Every recipe command succeeds, contains no braces/brackets, and produces the expected standalone definition. The separate [authoring transcript](../evidence/output-views/authoring.jsonl) retains the initial complete construction. |
| 2. Transfer and use | A separate authoring session imports, accepts, saves, activates, and exports the definition; both ordinary verbs run from the transferred standalone file and interface bundles. |
| 3. AGP parity | Both nonempty and empty connection/route tables match frozen AGP stdout byte for byte; cleaned display rows also match the independent jq projection. Uptime/TTL, selected routes, next hops, booleans, empty results, and hostile strings are covered. |
| 4. Native execution | Real CLI processes render with `PATH` set to an empty directory. The Rust suite invokes neither Bash nor jq nor column. |
| 5. Data/effect fidelity | Original receipt JSON remains intact. Typed projection preserves `9007199254740993`, `1.2300`, `1e400`, false, arrays, null, and missing-field distinctions; cell formatting affects display only. |
| 6. Retained rendering | After removing source files and omitting grants, `:render` preserves checkpoint bytes and revision and labels the output historical. Simulated results retain their runtime label. |
| 7. Failure behavior | Landed malformed-definition mutations reject before a session is created; input/work/padding/duration bounds reject explicitly. A failed presentation after a successful simulated edit retains `status: ok`, `mutation: applied`, its outcome, and an inspectable presentation error. `/dev/full` delivery failure preserves the invocation for later rendering. |
| 8. Compatibility and assembly | Old definitions serialize without new fields, legacy raw output retains exact number spelling, existing assembly suites pass, and a v3 component scopes view references while preserving its expression paths and resulting table. |

## Reference and implementation identities

The frozen fixtures, projected display rows, and tables came from AGP revision `da1a03a370f522bfb2a3afc66fb781b295cb6af7` before implementation.
The [oracle manifest](../evidence/output-views/agp-oracle.sha256) identifies their original bytes.
The [AGP source record](../research/command-output-views/AGP.md) identifies the original templates and renderer, and the unchanged upstream test suite passed thirteen tests during research.
The baseline CLI was source revision `543de9bb666b6ab4be45b9b069bf855d3771de80`; its captured [binary identity](../evidence/output-views/baseline-binary.sha256) and [exit](../evidence/output-views/baseline-exit.txt) record rejection of the new table option.

Reference acquisition used AGP's existing shell/jq tools.
The delivered application uses Rust expressions, `num-bigint` for exact bounded duration arithmetic, and `unicode-width` for cell layout.
Both libraries were already resolved in the original lockfile; this change adds them as direct dependencies without changing package versions.

## Corrections retained

The [first focused run](../evidence/output-views/first-native-tests.log) passed nine tests and failed two.
One test incorrectly expected inter-column padding growth from a single-column table; it now uses two columns, and the original failure is retained.
The other exposed an actual decoder defect: Serde's unit variant accepted extra fields for the plain format.
The [second run](../evidence/output-views/second-native-tests.log) isolated that accepted mutation; changing plain to a strict empty struct variant corrected it.
The [third run](../evidence/output-views/third-native-tests.log) passed all eleven focused tests.
Source review also corrected leading-zero handling near the duration digit bound and added a rounding-carry bound, with regression assertions in the final suite.

## Verification and limits

The [check ledger](../evidence/output-views/checks.tsv) records each contributor command's own exit code and corresponding logs.
The [verification record](../evidence/output-views/verification.json) identifies source/binary evidence and the measured commands.

| Required check | Measured result |
|---|---|
| Strict Clippy: normal, all features, scaffold | All three commands pass with warnings denied. |
| Normal application tests | 83 passed, including the eleven output-view tests. |
| Instrumented application tests | 100 passed, including fault and continuation cases. |
| Root and scaffold formatting | Both pass. |
| Scaffold tests | 13 passed, including documentation links, placement, and board/record consistency. |
| Generated documentation drift | 7 declared layers and 15 synchronized views. |
| Normal release build | Passes without instrumented features. |
| Literal consumer guide | Every shell block in `USE.md` passes under POSIX `sh`; [captured output](../evidence/output-views/guide-replay.log) includes both tables, transferred use with empty PATH, and retained-result rendering. |

The frozen fixture/output manifest is rechecked after implementation.
Earlier tracked evidence, approvals, and acceptance fixtures are unchanged.
Reference tables retain intentional trailing spaces produced by AGP's sanitization; those bytes are part of the oracle.

The layout matches the four frozen plain AGP tables; terminal color, width truncation, sorting/grouping, streaming, full jq semantics, arbitrary scripts, and network providers are outside this scope.
Numeric equality follows exact document token equality, not a new numeric comparison language.
Work and output bounds reject oversized presentations rather than promising unlimited result sizes.
Ephemeral invocation receipts last only for their process; use `--session` for recovery across launches.
Comparative agent effort, live endpoints, hosted CI for this change, and deployment are not measured by these local checks.
