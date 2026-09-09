# CLI-004 design audit

**Verdict: pass-with-guardrails**, before application implementation. This is the implementing author's design judgment, not independent certification.

Authority: [owner selection](../context/cli-004-approval.json). Design: [contract](CONTRACT.md), [acceptance task](acceptance/TASK.md). Canonical A3, A5, A7, M7, W14, AR3, and AR5 were read for this work. Existing [authoring](../authoring/AXIOM-AUDIT.md) and [composition](../composition/AXIOM-AUDIT.md) audits supply unchanged obligations.

## Fixed-intent survey bypass

The survey skill's fixed-intent path applies: the owner explicitly requested schema construction, a separate constraint role, OpenAPI authoring as data, and end-to-end completion. Those directions are already specified. No survey artifact is fabricated. Dialect, admission bounds, and implementation mechanics are engineering judgments tested against the task and external cases; the approval does not imply unlimited dialect coverage.

## Load-bearing obligations

| Principle | Design consequence and closeout observation |
|---|---|
| A1, state transparency | Attachment and exact policy are in headers and checkpoint; draft validity is explicit, never inferred from dirty state. |
| A2, specification | The authored snapshot governs validation. Changed schema data does not secretly switch roles. |
| A3, composition | Constraints depend on documents; the shared runtime owns revision and publication. The optional layer works with and without CLI composition. |
| A4, knowledge | Exact numbers, original intent, attachment, draft, and receipt survive reopen after source deletion. |
| A5, parity | Machine requests, terminal commands, and completion share admission and guidance. Compare actual process results. |
| A7, resilience | Bad attachments and rejected commits preserve checkpoint bytes. Exercise receipt replay and observed publication faults. |
| A8, integrity | Upstream positive and negative cases are retained unchanged; independently specified expected schema and OpenAPI documents precede implementation. |
| A9, failure validation | Count only fault cells whose marker proves the injection landed. Local tests are not deployment evidence. |
| A11/A12, focused context | Guidance exposes local types and children, paths identify failures, and incomplete/truncated guidance is labeled. |
| A13/A14, intent and learning | Complete the selected milestone; leave reuse economics and fresh-actor evaluation to CLI-005. Record corrections with their evidence. |

## Tensions and guardrails

Rejecting every invalid intermediate instance would prevent construction of required objects. The chosen transaction boundary permits drafts and enforces full validity on commit. A saved draft is not a certified instance.

General JSON Schema does not always permit complete finite suggestions. The validator remains the authority; a separate, conservative guidance projection reports its limits rather than inventing constraints. Test non-enumerable rules and branch-dependent children.

Exact numbers require more than preserving text in storage. The dependency probe observed exact bounds, decimal divisibility, exponent equality, and local reference error paths; acceptance must include these through the real CLI. Unsupported resource sizes reject explicitly.

Keep old profile declarations and checkpoint acceptance unchanged. Do not weaken receipt validation to accommodate attachment state. Test malformed digests, profile confusion, replay, read-only operations, and combined composition.

No unresolved design deviation requires owner input. The supported subset and local trials do not establish universal CLI coverage or the asymptotic product vision.

## Correction: fixture timing

The A8 row above describes expected documents as preceding implementation. More precisely, the acceptance task's required document contents and the unchanged upstream cases were recorded first. The JSON oracle files and literal construction traces were transcribed after the initial application implementation began. They derive from that earlier task, not from produced output. Their retained hashes establish the bytes tested; they do not establish earlier creation timestamps.
