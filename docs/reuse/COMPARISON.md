# Controlled authoring comparison results

**Status: all three methods passed the definition/runtime checks; file editing failed full staged continuation.**
These are measured results from the [retained release run](../evidence/cli-005/comparison/walkthrough/comparison.json), executed under the [approved contract](COMPARISON-CONTRACT.md).
Each method replayed a known recipe on a copy of the same prepared package.
The earlier [fresh-agent trial](FRESH-ACTOR-TRIAL.md) remains a separate observation.
This is the historical experiment before document import existed.
The subsequent [import results](../authoring/IMPORT-RESULTS.md) add and measure an explicit import method that passes the staged-state checks; the original methods and measurements below remain unchanged.

## Outcomes

| Method | Required definition and runtime | Full staged continuation | Total CLI input lines |
|---|---|---|---|
| Continue the draft | Pass | Pass | 77 |
| Edit exported JSON | Pass | Fail | 19 |
| Rebuild the candidate through commands | Pass | Pass | 199 |

All three produced the required definition, passed the unchanged schema, and exported the required interface with exact numbers, contexts, original intent, and simulation labels.
The evaluator exercised queue inspection, argument-sensitive pause behavior, exact quota changes, the preserved service inspection, and unbound-operation rejection through the actual CLI.
Each method retained the incompatible-declaration rejection, stale-request rejection, and failed initial commit.

The file method exported the unfinished candidate, made two literal edits, validated the edited file, and activated it.
Its candidate and accepted documents still contained the unfinished draft and original baseline.
The strict evaluator therefore reported `Worker definition differs from the fixed task or has not been committed`.
The [original continuation entry point](../../tools/continuation/trial_oracle.rs) still rejects this submission.
The artifact pass does not satisfy the original command-only construction requirement.

The application currently has no operation to import that file into the candidate.
File activation changes the active interface independently of the staged documents; it is not an import operation.
A future import or explicit conversion workflow would need its own contract and measured costs before full continuation parity could be claimed.
This observation applies to the tested workflow, not every possible integration of a text editor.

---

## Construction work and traffic

| Method | Construction CLI lines | Construction input bytes | Construction response bytes | External edit payload |
|---|---|---|---|---|
| Continue the draft | 57 | 1,620 | 64,364 | None |
| Edit exported JSON | 1 | 31 | 1,214 | Two literal edits, 994 recipe bytes |
| Rebuild the candidate | 179 | 4,990 | 196,971 | None |

The file method's construction line is `save completed.definition.json`.
Its [editor record](../evidence/cli-005/comparison/walkthrough/targeted-json-editing/package/external-edit/result.json) retains the 1,771-byte input document, 2,490-byte written document, exact before/after files, and recipe.
Separate [validation before editing](../evidence/cli-005/comparison/walkthrough/targeted-json-editing/package/external-validation-before.json) failed on the string `false`; [validation after editing](../evidence/cli-005/comparison/walkthrough/targeted-json-editing/package/external-validation-after.json) passed on the boolean.
Those validations read the respective document sizes again using the existing Rust schema checker.
Document sizes describe primary payloads, not aggregate filesystem I/O including hashing, verification, and evidence copies.

| Method | Total input bytes | Total response bytes | Submitted checkpoint bytes |
|---|---|---|---|
| Continue the draft | 2,129 | 121,844 | 15,363 |
| Edit exported JSON | 528 | 56,489 | 13,764 |
| Rebuild the candidate | 5,499 | 254,455 | 15,363 |

Totals include the shared controls and completion commands; evaluator probes are recorded separately.
Each definition is 2,490 bytes and each interface export is 7,795 bytes.
Definitions have the same structured values and exact numeric spelling; their object ordering and file hashes need not match.
The smaller file-method checkpoint reflects different staged contents and is not evidence of more efficient equivalent handover.

Continuing the draft avoided 122 construction commands compared with this literal rebuilding recipe.
That arithmetic describes these known command sequences; it does not estimate agent effort or establish an optimal strategy.
Rebuilding starts with an empty candidate inside the transferred session and reuses its policy and provenance.
It is not reconstruction of an entire session from nothing.

---

## Preparation and timing

The [shared preparation](../evidence/cli-005/comparison/walkthrough/shared/preparation.json) constructed the policy in 49 CLI inputs, the reusable source in 114, and the partial extension in 16.
Their recorded input volumes were 1,220, 3,157, and 538 bytes; response volumes were 37,900, 83,064, and 20,923 bytes.
Preparation took 1,989 ms, including CLI work, copying, hashing, and manifest creation.
Each method then copied and verified the same 21,357,216-byte immutable package, including its executables.
The preparation record's `not-run` trial fields describe that preparation stage; the enclosing comparison record contains subsequent execution verdicts.

| Method, in execution order | Package copy and checks, ms | Workflow wall time, ms | Method including evaluation, ms |
|---|---|---|---|
| Continue the draft | 269 | 1,318 | 1,621 |
| Edit exported JSON | 292 | 601 | 864 |
| Rebuild the candidate | 260 | 3,061 | 3,320 |

Workflow wall time includes package copying and method execution.
The experiment took 7,802 ms including shared preparation and all method evaluations.
These are nested measurements: do not add the columns, process durations, or receiver durations together.
Sub-millisecond editor and validator durations were recorded as zero at integer-millisecond resolution.
Compilation, recipe authoring, and observer documentation are outside this execution timer.
The methods ran sequentially once each without randomized order or cache controls.

Token use, human recipe-authoring effort, and comparative agent performance are unmeasured.
No economic advantage for equivalent full workflows is claimed while the file method fails staged continuation.
The observed whole-prototype reuse also does not establish isolated component composition.

---

## Verification and retained correction

The [final release trial suite](../evidence/cli-005/comparison/trial-tests-final.log) passed all six tests.
It checks the three outcomes and staged-state difference, rejects a valid but wrong external definition after activation, and rejects missing or ambiguous text anchors without partially writing the input file.
The existing continuation, receiver, transfer, and three semantic-mutation checks passed in that suite too.
The [documented workflow](COMPARISON-USE.md) was then executed separately; its [log](../evidence/cli-005/comparison/walkthrough.log) and [exit record](../evidence/cli-005/comparison/walkthrough.exit) retain the result used above.

**Correction: the first trial-suite run failed in the new mutant fixture.**
It copied an existing interface export into the mutant directory, so the product correctly rejected the changed export with `EXPORT_EXISTS` before the intended oracle check was reached.
That first attempted mutant check is invalid as oracle-rejection evidence.
The fixture now exports to a new destination and proves the mutation reached the active interface before checking rejection.
The [original failed run](../evidence/cli-005/comparison/trial-tests.log) and its [failed-trial artifacts](../evidence/cli-005/comparison/regression/failed-trials/) remain available beside the passing result.

The [archive mapping](../evidence/cli-005/comparison/archive-map.tsv) records byte-preserving text copies from the release run.
Copied task and start documents use `.txt` extensions to retain their original bytes without treating package-relative links as project-document links.
Executables remain in the original run directory; their hashes are retained in the [source inventory](../evidence/cli-005/comparison/source-inventory.sha256).
The [archive integrity check](../evidence/cli-005/comparison/archive-integrity.log) verifies the retained evidence files.
The [runtime continuity check](../evidence/cli-005/comparison/runtime-continuity.log) confirms unchanged application and Cargo sources against the earlier record.
Current trial sources and release executables have [separate hashes](../evidence/cli-005/comparison/source.sha256) and [binary hashes](../evidence/cli-005/comparison/release-binaries.sha256).
This turn changed Rust trial tooling, tests, recipes, and project records; it added no runtime operation or checkpoint format.

The [design audit](COMPARISON-AUDIT.md) guardrails are discharged by the paired verdicts, actual method traces, preserved failed attempt, adversarial checks, and scoped costs above.
CLI-005 remains active for full-workflow comparability and the concrete component-composition boundary recorded on the [board](../BOARD.md).

---

## Mechanics, rationale, and consequence

### Mechanics

Compare the required artifacts through one task oracle, then report each method's original staged-continuation verdict and actual work separately.

### Rationale

The experiment identifies both a measurable benefit of continuing existing structure and a missing boundary for importing external work.

### Consequence of violation

Reading the artifact pass as complete handover, or command counts as agent savings, would conceal the exact difference this comparison exposed.
