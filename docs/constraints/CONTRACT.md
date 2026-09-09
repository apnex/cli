# Schema constraints contract

Selected work: [CLI-004](../BACKLOG.md#cli-004), with [owner approval](../context/cli-004-approval.json).
This contract was recorded before application implementation. The [acceptance task](acceptance/TASK.md) fixes the intended behavior; the [audit](AXIOM-AUDIT.md) records the design judgment.

## Explicit document roles

Start with `--constraints`, independently of `--compose`. Ordinary authoring can construct any JSON document, including incomplete schemas and API descriptions. A document never becomes a constraint merely by containing schema keywords.

`constrain [source]` snapshots the whole candidate, or a bounded local JSON file, as the active schema. It validates the schema before changing the attachment. It preserves candidate, accepted document, and authoring context. Editing the source later does not change the snapshot. `unconstrain` removes it explicitly. Both use the existing revision, publication, and replay transaction.

`schema` exposes the exact attached schema text, digest, and interpretation. Session headers expose the attachment identity and mode. Reopen needs no source file. Detachment and discard are different: discard restores the accepted document and retains the attachment.

## Draft and acceptance policy

All structurally valid primitive edits remain available, including edits that temporarily violate a schema. No defaults are inserted and no values are coerced. `validate [candidate|accepted]` returns validity and bounded findings without mutation. A constrained `commit` validates the entire candidate first. Failure retains the complete acknowledged checkpoint, context, revision, attachment, receipt, and editable draft.

Changing constraints does not retroactively certify the accepted baseline. `validate accepted` evaluates it under the current attachment. `save` continues to export the candidate, including unfinished drafts; it is not a certification operation. Acceptance is through `commit`.

## Interpretation

The first policy is `local-2020-12-v1`, using JSON Schema Draft 2020-12. An absent `$schema` selects that dialect; an explicit declaration must be `https://json-schema.org/draft/2020-12/schema` (an optional trailing `#` is accepted).

Supported assertion/applicator keywords are `type`, `enum`, `const`, `properties`, `required`, `additionalProperties`, `patternProperties`, `propertyNames`, `dependentRequired`, `dependentSchemas`, `items`, `prefixItems`, `contains`, `minContains`, `maxContains`, `minItems`, `maxItems`, `uniqueItems`, `minProperties`, `maxProperties`, `minimum`, `maximum`, `exclusiveMinimum`, `exclusiveMaximum`, `multipleOf`, `minLength`, `maxLength`, `pattern`, `allOf`, `anyOf`, `oneOf`, `not`, `if`, `then`, `else`, `unevaluatedProperties`, and `unevaluatedItems`. Boolean schemas, `$defs`, and local JSON Pointer `$ref` are supported.

`title`, `description`, `default`, `examples`, `deprecated`, `readOnly`, `writeOnly`, `$comment`, and `format` are annotations. In particular, `format` does not assert validity. Annotation values are data, not recursively interpreted as schemas.

Admission rejects all other keywords at schema locations, alternate dialects, anchors, resource IDs, dynamic references, external references, unresolved references, references into non-schema data, and cyclic reference expansion. This is an explicit application subset, not a claim that those features are invalid JSON Schema. Regular expressions use the validator's linear-time regex engine; unsupported syntax is rejected at attachment. HTTP and filesystem retrieval are disabled in the dependency and the builder.

Interpretation delegates to pinned `jsonschema` 0.55.0 with arbitrary precision and no default features. Exact numbers enter it by parsing preserved JSON text. Constraint evaluation bounds number tokens to 1,024 bytes and exponent magnitude to 4,096; values outside this range remain authorable but validation reports `LIMIT_EXCEEDED`. No rounding substitutes for evaluation.

Schema admission is bounded to 64 KiB, 256 schema locations, and 4,096 visits in expanded reference/applicator traversal with maximum expansion depth 64. Existing document and transport bounds continue to apply. Validation returns at most 100 findings, states whether truncated, and never calls a truncated invalid report valid. Findings carry instance and schema JSON Pointers, a keyword, and a bounded message.

## Guidance and completion

`guide [path]` describes schema locations applicable to a typed document path, including absent final properties, expected types, literal enum/const suggestions, required fields, and declared children. Direct properties, array items/prefix items, and local references provide structural suggestions. Completion consumes the same guidance and includes declared absent properties as well as existing editable children.

Guidance is advisory. Combinators, pattern-based fields, dependencies, and evaluation-dependent rules can limit its completeness; the result reports that limitation. A suggestion never promises the resulting whole document is valid. `validate` and `commit` use the validator, not the completion projection.
The `complete` field describes the direct structural projection. It does not certify suggested instances. Reference siblings and other conjunctive rules report incomplete projection when suggestions do not solve their intersection.

## Persistence and compatibility

Plain authoring and composition keep their existing format 1 and 2 definitions and hashes. Constraints alone use checkpoint format 3, `bootstrap-json-constraints-v1`; composition plus constraints use format 4, `bootstrap-cli-constraints-v1`. The new declaration digest hashes the selected base profile's hexadecimal digest, a newline, and the exact constraints extension bytes. Profile mismatch rejects reopen without rewriting it.

The attachment stores exact schema values, dialect, policy, and SHA-256 of compact document text. Reopen rechecks admission and digest, and checks state receipts against the attachment header. Attachment and commit use one existing publication path. No session migration, network execution, or API invocation is introduced.

## Source basis

The [JSON Schema core](https://json-schema.org/draft/2020-12/json-schema-core) and [validation specification](https://json-schema.org/draft/2020-12/json-schema-validation) define the dialect. The [Rust validator documentation](https://docs.rs/jsonschema/0.55.0/jsonschema/) describes precision and offline options; the retained dependency probe measures selected behavior locally. The [OpenAPI 3.1.1 specification](https://spec.openapis.org/oas/v3.1.1.html) supplies the representative description structure. Local acceptance establishes the stated subset, not complete standards conformance or workflow superiority.
