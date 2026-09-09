# Document import implementation and results

**Measured result: an authored CLI specification can be exported, imported into a separate session, activated, used, and resumed.**
The [owner's agreement](../context/cli-005-import-approval.json) selected this increment.
The [contract](IMPORT.md) and [design audit](IMPORT-AUDIT.md) preceded implementation.
The [walkthrough](IMPORT-USE.md) gives the executed commands; [verification](../evidence/document-import/verification.json) records their sources and evidence.

## Implemented behavior

`import <source>` replaces the whole candidate through the shared Rust transaction, retaining exact JSON values and returning authoring focus to root.
The accepted document, attached schema, active interface, and simulated state retain their existing lifecycle.
Schema-invalid imported drafts remain editable; `commit` validates them, and `activate` independently selects the runnable interface.
The import result records actual changed paths and the exact source-byte digest and length.
Reopening and replaying its last receipt do not reread the source file.

The operation is declared in [bootstrap data](operations.json), handled by the existing authoring registry, and exposed through shared help, completion, terminal input, and machine requests.
Receipt validation now checks the import result's shape, source, digest syntax, byte bound, paths, and root context without requiring the file to exist.
The checkpoint formats and profile identifiers are unchanged; the operation declaration's digest changes.
Previous checkpoints stay bound to their original declaration, so the prior executable and declaration were [preserved](../evidence/document-import/before/runtime.sha256).

The separate consumer uses `--compose` with `activate`, `tree`, `enter`, and `invoke`.
There is no new `run` command or runtime-only presentation in this increment.
The demonstrated bindings are simulated or explicitly unbound.

---

## Separate author and consumer

The producer constructed the complete service-catalog specification through the existing typed command recipe.
It exported JSON without manually written container literals.
Separate terminal and machine consumers imported that file and were checked against the unchanged expected definition.
Both committed, activated, inspected the tree, invoked exact quota changes, exported the interface, and resumed after source removal.
The active quota retained token `1e400` while the authored definition retained its original values.

The independently executed [walkthrough log](../evidence/document-import/walkthrough.log) includes the same separate-session workflow.
Its author and consumer document exports compared byte-for-byte.
The source file was renamed before reopening, and the receiver still printed its tree and invoked inspection.
The retained [author trace](../evidence/document-import/walkthrough/author/author.jsonl), [consumer trace](../evidence/document-import/walkthrough/consumer/use.jsonl), and [resumption trace](../evidence/document-import/walkthrough/consumer/resumed.jsonl) contain the actual requests' results.

---

## Updated controlled comparison

The [current release comparison](../evidence/document-import/comparison/comparison.json) retains the three prior methods and adds edited-file import.
All four passed the common definition/runtime outcome checks.

| Method | Construction CLI inputs | Total CLI inputs | Full staged-state checks | External work |
|---|---|---|---|---|
| Continue the existing draft | 57 | 77 | Pass | None |
| Edit a file and activate it directly | 1 | 19 | Fail | Two text edits |
| Rebuild the candidate | 179 | 199 | Pass | None |
| Edit a file, import, commit, and activate | 3 | 23 | Pass | Two text edits |

The import method's construction inputs export the unfinished draft, import the edited file, and restore the task's authoring context.
Its full trace contains 598 input bytes and 61,628 response bytes.
The two external edits still require a 994-byte recipe, consume a 1,771-byte input document, and write a 2,490-byte edited file.
External validation reads the before and after files separately; the import receipt records its own 2,490-byte source read.
Edited source and final canonical export have separate filenames, so both representations remain available.
The submitted checkpoint is 15,363 bytes, definition 2,490 bytes, and interface export 7,795 bytes.

Continuing and rebuilding used 2,129 and 5,499 CLI input bytes respectively; their response volumes were 122,290 and 254,901 bytes.
The activation-only file method used 528 CLI input bytes and 56,935 response bytes, but retained unfinished staged documents.
These are scoped payload volumes, not aggregate filesystem I/O or token counts.

Shared preparation constructed the policy in 49 CLI inputs, the reusable source in 114, and the partial extension in 16.
It took 1,546 ms; the complete four-method experiment took 7,319 ms including preparation and evaluation.
The import method's workflow wall time was 526 ms, including its package copy and checks.
Nested process, receiver, workflow, and experiment times overlap and must not be added.
This is one sequential execution of each known recipe; compilation, recipe-authoring effort, and agent reasoning are outside those timers.

The original continuation evaluator also accepted the imported staged state in the regression test.
The [historical failed file method](../reuse/COMPARISON.md) remains retained, and direct file activation continues to fail those staged checks.
Staged-state parity is now measured for an explicit import workflow.
External text editing remains a different authoring method; no comparative agent, token, or full economic advantage is claimed.

---

## Verification and corrections

The [instrumented application suite](../evidence/document-import/application-tests.log) passed 63 tests, including six import tests and six continuation tests.
The [normal-build suite](../evidence/document-import/normal-tests.log) passed 49 tests, including the check that normal builds ignore test fault controls.
The import corpus covers exact values, root arrays/scalars, malformed input, repeated decoded keys, invalid Unicode, size/depth limits, missing/nonregular files, and rejected edits preserving checkpoint bytes.
Integration tests check constrained repair, unchanged active interfaces, declaration-driven discovery, receipt corruption, source-free replay, and publication failure/uncertainty.
These are local executor-run checks, not independent certification or new fresh-agent measurements.

The [failing-before test](../evidence/document-import/failing-before.log) reached `UNKNOWN_OPERATION` on import before the handler was added.
The [first integration run](../evidence/document-import/integration.log) passed the round trip and all four comparison methods but failed one test assertion about uncertain-process exit.
**Correction:** the new test initially expected exit success after `PERSISTENCE_UNCERTAIN`; the existing runtime and contract require a nonzero exit.
The assertion was corrected; runtime exit behavior was not changed.
That first run did not reach its subsequent source-removal/reopen assertions.
The final suite reached and passed them, with both injected publication boundaries recorded before their outcomes were evaluated.

The [archive mapping](../evidence/document-import/comparison/archive-map.tsv) and [integrity record](../evidence/document-import/archive-integrity.log) retain byte-preserving text evidence.
Executables remain in the original run directory, with hashes in the source inventory and current [binary record](../evidence/document-import/release-binaries.sha256).
The [source record](../evidence/document-import/source.sha256) identifies the tested implementation and acceptance files.

All import-audit guardrails have corresponding checks above.
CLI-005 remains active for independently reusable component assembly, state ownership, dependencies, and collisions.
Those are the next product boundaries; the previously measured import gap is resolved within this complete-document scope.

---

## Mechanics, rationale, and consequence

### Mechanics

Use one declared import handler, test the actual separate-session consumer, and retain all comparison methods with their scoped costs and verdicts.

### Rationale

An exported specification can now reenter both the editing workflow and configured use without reconstructing its document through individual commands.

### Consequence of violation

Reporting only interface activation would conceal incomplete staged work; reporting only command counts would conceal external editing and unknown actor costs.
