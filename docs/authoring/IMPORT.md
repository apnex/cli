# Document import contract

**Status: implemented and verified locally under CLI-005.**
The owner's [agreement and round-trip question](../context/cli-005-import-approval.json) select complete-document import into the candidate.
This extends the [authoring contract](SESSION-CONTRACT.md) without changing acceptance or activation semantics.
The [implementation results](IMPORT-RESULTS.md) retain measured acceptance and the [walkthrough](IMPORT-USE.md) gives executable use.

## Operation and state

`import <source>` explicitly replaces the entire candidate with one JSON document from a local file.
It does not merge members or select a subtree.
The machine operation is `import` with one required string argument, `source`.
It is a state change requiring the observed revision and is not permitted in a batch.
The loaded operation declaration owns its name, arguments, help, terminal binding, and native handler registration.

The operation parses into the existing exact-value document model before replacing the candidate.
It resets authoring context to the root, preserves the accepted baseline and attached constraints, and leaves the active interface and its simulated state intact.
An imported document can remain schema-invalid while it is being repaired; `commit` enforces the attached schema.
`activate` remains the explicit operation that makes a valid CLI definition usable.

The successful response includes actual changed paths, the source path exactly as supplied, the SHA-256 of the exact bytes read, and their length.
The source digest describes the bytes consumed, not a later filesystem read or an authenticated origin.
Number tokens retain their exact spelling; formatting and object-member order follow the existing document contract.
Receipt replay returns the persisted import outcome without rereading a changed or missing source file.

---

## File and failure boundary

Read a bounded local regular file through the existing storage reader.
The source byte limit is 2 MiB, including formatting; compact document, scalar, and nesting limits remain those in the authoring contract.
Missing or unsupported file targets report `IMPORT_FAILED`, malformed JSON or UTF-8 reports `INVALID_VALUE`, and exceeded bounds report `LIMIT_EXCEEDED`.
Duplicate decoded keys, invalid Unicode, trailing input, and non-JSON numbers reject.
Failed argument, revision, parse, or limit checks preserve checkpoint bytes, candidate, accepted state, contexts, and active interface.
Checkpoint publication and uncertain-response recovery use the existing transaction and receipt boundary.

An interface bundle is ordinary JSON when imported as a document.
Use its existing `activate <bundle>` operation to restore interface state; import performs no implicit bundle extraction or activation.
Adding a registered operation changes the declaration digest.
Preserve the prior binary and declaration for old checkpoints; do not rewrite historical checkpoints or receipts to fit the new declaration.
Exported definitions remain the transfer boundary for starting a new session with the new surface.

---

## Acceptance predicate

| Consumer or failure | Required observation |
|---|---|
| Separately used CLI specification | Author the catalog through commands, export it, import into a different session, inspect, commit, activate, print its tree, invoke its mocks, and reopen successfully. |
| Exact structured data | Numbers, unusual keys, root scalars, arrays, and nested values survive import, further editing, export, and reopen. |
| Existing draft and interface | Import resets document focus while preserving accepted data, schema, active definition, and mock state until explicit later operations. |
| Constrained imported draft | Import accepts editable invalid data; commit rejects it without changing state; typed repair permits commit. |
| Rejection and retry | Missing/malformed/oversized input and stale requests preserve bytes; replay after source removal returns the same accepted receipt. |
| Persistence failure | Import uses the existing failure and uncertain-publication boundaries; reopen resolves its receipt without requiring its source. |
| Discovery and parity | Help, completion, terminal commands, and machine requests agree on the declared import operation and state transition. |
| External editing comparison | Retain the original file-activation method and add an edited-file import method that meets the original staged-state checks with all extra work counted. |

The original authoring corpus and previous comparison evidence remain historical inputs.
New acceptance cases supplement their coverage rather than modifying old expected outcomes.
A passed staged-state comparison does not turn external text editing into command-only construction or measure comparative agent performance.

---

## Mechanics, rationale, and consequence

### Mechanics

Parse one explicit file, stage its document through the shared transaction, and retain independent acceptance and activation boundaries.

### Rationale

Existing JSON and exported specifications become editable starting material for a separate session.

### Consequence of violation

Implicit activation, lossy parsing, or receipt rewriting would make a portable specification change meaning when it reenters the workflow.
