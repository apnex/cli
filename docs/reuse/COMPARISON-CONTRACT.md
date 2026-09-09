# Controlled authoring comparison

**Status: selected engineering experiment, specified before implementation and execution.**
This records the historical three-method experiment before document import was implemented.
The later [import contract](../authoring/IMPORT.md) adds a fourth method while preserving these original methods and predicates.
The owner's [approval](../context/cli-005-comparison-approval.json) selects the comparison following the first fresh-agent trial.
The [original task](acceptance/TASK.md) and [catalog task](../composition/acceptance/TASK.md) remain unchanged.

## Methods and controls

| Method | Work performed |
|---|---|
| CLI continuation | Repair the supplied scalar and build the missing queue commands through typed operations. |
| Targeted JSON editing | Export the unfinished candidate, apply two literal text edits to that file, validate the actual edited file, and activate it through the existing file activation operation. |
| CLI reconstruction | Clear only the candidate, construct the complete required definition through literal typed commands, then commit and activate it. |

All three runs receive byte-identical copies of one prepared checkpoint and the same runtime, declarations, schema snapshot, and tasks.
Each performs the same initial discovery, incompatible-declaration probe, stale-request rejection, and invalid-draft commit.
Source, policy, and partial-draft construction are measured as shared preparation, with their full costs retained.
Reconstruction starts from an empty candidate inside the transferred session; it does not reconstruct the policy or session metadata.
The methods run sequentially once each, with known literal recipes; timings describe these executions and do not estimate a population or control cache effects.

The fixed-intent survey bypass applies: the owner selected the comparison and its outcomes.
The choice of these controlled recipes is an engineering decision, not a claim about the best possible editor or agent strategy.

---

## Acceptance and the observed import gap

The runtime can activate an external definition file, but its operation declaration has no document-import operation.
Container constructors create empty containers and cannot import serialized trees.
These observations come from the actual launcher, registered authoring handlers, constructor codec, and file activation handler.

Two verdicts must remain separate:

| Verdict | Predicate |
|---|---|
| Definition and runtime outcome | The saved definition exactly satisfies both tasks, including number spelling; the unchanged policy validates it; the active/exported interface has that definition and the required final state, context, labels, and original intent; runtime probes pass. |
| Full staged continuation | The original continuation checks also hold, including candidate and accepted documents matching the completed definition, unchanged constraints, and the recorded continuation obligations. |

The second verdict is never weakened to accommodate file editing.
The expected file-editing limitation is recorded as a failed full-continuation verdict, even if its definition and runtime outcome pass.
No checkpoint injection, fabricated receipt, new import command, or typed reconstruction of the edited file may hide that gap.
Targeted JSON editing is permitted only for the comparison's exported document; this is not reported as satisfying the original task's command-only method.

The existing task oracle supplies the shared semantic checks and runtime probes.
Its strict continuation entry point retains the original requirements; the comparison exposes an artifact-outcome entry point and reports the strict verdict alongside it.
The accepted source, expected definition, and historical actor evidence remain unchanged.

---

## Evidence and stopping conditions

Retain raw CLI requests, responses, failures, checkpoint and export bytes, edit recipe, exact before/after document bytes, validation results, and method order.
Report CLI input bytes, external edit payload, document reads/writes, process and method wall times, package preparation, and artifact volumes separately.
Overlapping timers and repeated representations are not added into a misleading total.
Tokens, human authoring effort, and comparative agent performance remain unmeasured.

The text editor applies literal edits only when each expected fragment occurs exactly once, and checks all edits before writing.
Missing or ambiguous fragments must reject without publishing a partial edit.
Both CLI recipes must contain no raw JSON containers.
Unexpected failure ends that method, records the failure and available trace, and does not erase the other methods' results.
Existing run directories cannot be overwritten.

Validation must exercise all three methods through the real binary, retain the file-import gap, reject a valid but wrong external definition, and prove that missing or ambiguous edit anchors do not modify their input.
No comparative saving for equivalent complete workflows may be claimed while the file method fails staged continuation.
General component composition remains outside this experiment.

---

## Mechanics, rationale, and consequence

### Mechanics

Replay fixed methods on independent copies, check their common artifacts through one oracle, and retain the original full-continuation verdict separately.

### Rationale

The comparison measures concrete construction work while allowing a missing capability to appear as an outcome.

### Consequence of violation

Mixing fresh-agent effort with scripted replay, omitting external edit bytes, or hiding the import gap would create an unsupported workflow-advantage claim.
