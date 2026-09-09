# Component assembly results

**Status: implemented and verified locally for the selected services/queues journey.**
The [approval](../context/cli-005-assembly-approval.json), [contract](CONTRACT.md), [task](acceptance/TASK.md), and [pre-implementation audit](AXIOM-AUDIT.md) retain the scope.
The [verification record](../evidence/component-assembly/verification.json) carries commands, identities, limits, and results.
This completes the selected local CLI-005 work; it is executor-run evidence, not independent certification or a new fresh-agent measurement.

## Implemented behavior

The declared assemble operation reads the candidate manifest and its local component files.
One Rust resolver validates dependencies and namespaces, scopes mock paths, embeds the sources, and produces an assembled definition.
Candidate replacement, commit, activation, and invocation keep their existing separate lifecycles.
Reopening, activation, and interface transfer validate the executable content against embedded components without opening the original paths.
The global simulated document must retain exactly the declared component scopes.

Plain definitions keep their prior format and whole-state behavior.
Assembled definitions use the new format with component snapshots and source observations.
The composition declaration digest changes; previous checkpoints remain bound to their matching runtime/declaration pair.
The prior runtime pair is identified in the [preservation record](../evidence/component-assembly/before/runtime.sha256).

## Measured acceptance

| Check | Observation |
|---|---|
| Full instrumented application suite | 69 tests passed, including six component-assembly tests and the existing authoring, composition, schema, import, and continuation cases. |
| Normal application suite | 54 tests passed, including five component tests; explicit fault tests belong to the separate instrumented run. |
| Paired construction and transfer | Both presentations authored the component documents through commands, assembled equivalent exact documents, and exercised a separate consumer. |
| Dependency/source/collision rejection | Twenty negative cases ran through both presentations; all forty rejected without changing checkpoint bytes. |
| Component state ownership | Service inspection stayed unchanged after queue mutation; service quota mutation left queue state unchanged. Root reads, assignments, scalar/array replacements, unusual keys, and nested contexts were exercised. |
| Embedded-source integrity | Applied executable-path and snapshot mutations rejected activation; missing runtime scopes rejected interface import and checkpoint reopening. |
| Recovery | Last-receipt replay and reopening succeeded without source files. Both injected publication boundaries were observed before their failure or uncertainty outcomes were checked. |
| Literal operator workflow | Every command in the [walkthrough](USE.md) ran successfully, including file comparisons, unchanged service reads, exact numeric resumption, and source removal. |

The [application log](../evidence/component-assembly/application-tests.log), [normal log](../evidence/component-assembly/normal-tests.log), and [walkthrough log](../evidence/component-assembly/walkthrough.log) retain the outcomes.
The [archived walkthrough](../evidence/component-assembly/walkthrough/) contains source exports, manifest, assembled definition, sessions, interface export, and event streams.
The archived bytes were compared against the original run.

## Workflow volume

These are observations from one literal scripted walkthrough.
Responses correspond to its successful CLI inputs; source construction, assembly, use, and resumption remain visible separately.

| Phase | CLI responses | Output bytes |
|---|---:|---:|
| Services component authoring | 60 | 42,757 |
| Queues component authoring | 54 | 38,527 |
| Manifest authoring and assembly | 18 | 13,644 |
| Separate consumer use | 14 | 15,947 |
| Consumer resumption | 3 | 7,208 |

The [machine-readable measurements](../evidence/component-assembly/walkthrough-metrics.jsonl) retain operation counts and zero error responses.
The two source recipes contain 3,135 and 2,724 input bytes; the manifest/assembly input contains 448 bytes, consumer use 221 bytes, and resumption 43 bytes.
The component exports contain 914 and 773 bytes, the manifest 230 bytes, assembled definition 3,908 bytes, interface export 9,466 bytes, and final consumer checkpoint 18,794 bytes.
Embedded snapshots account for intentional duplication in the portable artifact.
These measurements do not establish token savings, actor effort, or a comparative performance advantage.
Compilation and recipe-authoring effort are not inferred from CLI response counts.

## Corrections retained

The first test source named a nonexistent loader method and did not compile; the [compile record](../evidence/component-assembly/compile-first.log) is not behavioral failure evidence.
The first recipe used the root object's empty key instead of the root token; the [recipe record](../evidence/component-assembly/recipe-first.log) rejected its comparison with the unchanged expected component.
After those fixture corrections, the [failing-before run](../evidence/component-assembly/failing-before.log) reached the intended UNKNOWN_OPERATION rejection before assembly existed.

The [first integration run](../evidence/component-assembly/integration.log) passed five tests and failed the root-array fidelity assertion.
**Correction:** that fixture decoded its intended literal through generic JSON values, changing the exponent spelling to 1e+400 before supplying it to the CLI.
The corrected fixture uses exact document values; the application behavior was unchanged by that repair.
The final log records the generic encoding observation and passes the exact source literal through assembly and invocation.
All initial logs and test-source snapshots remain available.

## Scope and future consumers

The resolver supports bounded local components, one instance per source identity, acyclic presence requirements, namespaces, and explicit simulation.
Nested assembly, shared mutable state, schema merging, dependency version negotiation, and registries need a concrete consumer before extending the contract.
No new actor was asked to assemble components in this increment.
The earlier fresh actor completed its original whole-prototype task; those results are retained separately.
Real hooks and OpenAPI-derived invocation remain the held CLI-006 and CLI-007 work.

## Mechanics, rationale, and consequence

### Mechanics

Preserve source components, derive and check one assembled interface, and compare observed component behavior across transfer and recovery.

### Rationale

The construction workflow can now assemble reusable pieces while retaining the meaning and requirements of each piece.

### Consequence of violation

A portable file or combined tree alone could conceal changed behavior, missing provenance, or a dependence on source files that the receiver never received.
