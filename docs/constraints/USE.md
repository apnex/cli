# Construct schemas and author constrained JSON

The optional `--constraints` layer adds `constrain`, `unconstrain`, `schema`, `guide`, and `validate`. It works independently of `--compose`; both flags can be used together. The [contract](CONTRACT.md) states the supported Draft 2020-12 subset and bounds.

## Build the schema through commands

Run from the repository root with the [authoring prerequisites](../authoring/USE.md#build-and-start):

```sh
cargo build --locked --bin cli
cli_project_root="$PWD"
cli_constraints_dir="$(mktemp -d "$cli_project_root/target/constraints-demo-XXXXXX")"
(
  cd "$cli_constraints_dir"
  "$cli_project_root/target/debug/cli" \
    --definition "$cli_project_root/docs/authoring/operations.json" \
    --constraints --session schema.session.json --create \
    --intent-file "$cli_project_root/docs/constraints/acceptance/TASK.md" \
    --commands < "$cli_project_root/docs/constraints/acceptance/schema.commands" \
    > schema-construction.jsonl
  "$cli_project_root/target/debug/cli" \
    --definition "$cli_project_root/docs/authoring/operations.json" \
    --constraints --session schema.session.json --commands \
    > schema-save.jsonl <<'CLI'
save service.schema.json
CLI
)
```

The [construction trace](acceptance/schema.commands) uses navigation and typed constructors to create a service schema with required fields, an enum, unique tags, and a referenced port definition. It does not load the separate [expected schema](acceptance/service.schema.json). The resulting `service.schema.json` is authored output. It remains ordinary data until explicitly attached.

## Attach it to an independent instance

Using the same shell variables:

```sh
(
  cd "$cli_constraints_dir"
  "$cli_project_root/target/debug/cli" \
    --definition "$cli_project_root/docs/authoring/operations.json" \
    --constraints --session instance.session.json --create \
    --intent-file "$cli_project_root/docs/constraints/acceptance/TASK.md" \
    --commands > instance-attachment.jsonl <<'CLI'
constrain service.schema.json
guide
CLI
  "$cli_project_root/target/debug/cli" \
    --definition "$cli_project_root/docs/authoring/operations.json" \
    --constraints --session instance.session.json --commands \
    < "$cli_project_root/docs/constraints/acceptance/instance.commands" \
    > instance-authoring.jsonl
)
```

The [instance trace](acceptance/instance.commands) deliberately stages port `5432` as a string. That edit succeeds; the first `commit` reports `SCHEMA_VIOLATION` at instance `/port`, schema `/$defs/port/type`. The trace corrects it with `set /port number 5432` and commits successfully. Every failed operation preserves the saved draft and revision. A command stream continues after a rejected operation, so inspect its response statuses; process exit alone does not certify every command.

Open the resulting session interactively:

```sh
(
  cd "$cli_constraints_dir"
  "$cli_project_root/target/debug/cli" \
    --definition "$cli_project_root/docs/authoring/operations.json" \
    --constraints --session instance.session.json
)
```

Enter these CLI commands:

```text
schema
guide /port
guide /mode
complete
validate
show
```

The accepted document is `{"name":"catalog","port":5432,"mode":"production","tags":["public"]}`. This is displayed output; it was not entered as a JSON literal. Tab suggests missing schema fields after a path prefix, `number` for port, and enum literals after `set /mode string `. Guidance is advisory and reports when its structural suggestions are incomplete. Whole-document validation decides whether a commit can succeed.

## Author an OpenAPI description

```sh
(
  cd "$cli_constraints_dir"
  "$cli_project_root/target/debug/cli" \
    --definition "$cli_project_root/docs/authoring/operations.json" \
    --constraints --session openapi.session.json --create \
    --intent-file "$cli_project_root/docs/constraints/acceptance/TASK.md" \
    --commands < "$cli_project_root/docs/constraints/acceptance/openapi.commands" \
    > openapi-construction.jsonl
  "$cli_project_root/target/debug/cli" \
    --definition "$cli_project_root/docs/authoring/operations.json" \
    --constraints --session openapi.session.json --commands \
    > openapi-save.jsonl <<'CLI'
save openapi.json
CLI
)
```

The [OpenAPI trace](acceptance/openapi.commands) constructs GET `/services`, its JSON response schema, and a reusable Service component. `/paths/~1services` addresses the literal `/services` key; response `200` remains an object key. The result describes an API. No schema attachment or API invocation occurs during this construction. The fixture is checked against the [predefined document](acceptance/openapi.expected.json); this is not a full OpenAPI conformance validator.

## Controls and continuation

| Command | Result |
|---|---|
| `constrain` | Admit the whole candidate as an immutable schema snapshot. |
| `constrain file.json` | Admit an explicit local schema file without replacing the candidate. |
| `unconstrain` | Detach the schema; preserve both documents and context. |
| `schema` | Inspect exact schema text, digest, and interpretation. |
| `guide [path]` | Inspect local rules, required fields, types, literals, and completeness limits. |
| `validate [candidate\|accepted]` | Read full-document validity and up to 100 findings. |
| `commit` | Validate the complete candidate before accepting it. |
| `discard` | Restore the accepted document while retaining the attachment. |
| `save file.json` | Export the candidate, including an unfinished draft. |

The attachment is stored inside the checkpoint. Reopen does not need the source file. Editing or removing that file does not update the active schema; attach again to change it. `constrain` can attach a schema that the current candidate or accepted baseline does not satisfy. Inspect both with `validate` and `validate accepted`.

Use the same profile flags and exact operation declaration to reopen. Constraints alone use checkpoint format 3; adding composition uses format 4. Existing ordinary authoring and composition sessions retain their formats and do not migrate implicitly.

## Machine requests and evidence

Use `--machine` with the [existing request envelope](../authoring/SESSION-CONTRACT.md#request-and-result-contracts). `constrain` and `unconstrain` require `expected_revision`. `schema`, `guide`, and `validate` omit it. `constrain` accepts an optional string `source`; `guide` accepts a typed `path`; `validate` accepts `view`, defaulting to `candidate`. The complete contracts are also available through `help`.

Session headers expose `constraint_mode` and the active attachment's dialect, policy, and digest. `validate` returns `result.validation`; failed commits return `error.validation` with the same finding shape. Findings include `instance_path`, `schema_path`, `keyword`, and message truncation status. A report with truncated findings still reports invalidity.

The [implementation record](CLI-004.md) distinguishes measured local tests, upstream examples, and remaining limitations. Preserve checkpoint and export files you intend to keep; [removal guidance](../authoring/USE.md#verify-and-remove) covers local build output and session locks.
