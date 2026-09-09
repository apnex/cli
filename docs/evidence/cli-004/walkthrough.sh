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
(
  cd "$cli_constraints_dir"
  "$cli_project_root/target/debug/cli" \
    --definition "$cli_project_root/docs/authoring/operations.json" \
    --constraints --session instance.session.json
)
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
printf '%s\n' "$cli_constraints_dir" > "$cli_project_root/docs/evidence/cli-004/demo-directory.txt"
