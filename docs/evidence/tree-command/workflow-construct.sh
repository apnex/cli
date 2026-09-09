cargo build --locked --bin cli
cli_project_root="$PWD"
cli_platform_dir="$(mktemp -d "$cli_project_root/target/platform-demo-XXXXXX")"
(
  cd "$cli_platform_dir"
  "$cli_project_root/target/debug/cli" \
    --definition "$cli_project_root/docs/authoring/operations.json" \
    --compose --session session.json --create \
    --intent-file "$cli_project_root/docs/composition/examples/platform/TASK.md" \
    --commands < "$cli_project_root/docs/composition/examples/platform/platform.commands" \
    > construction.jsonl
)
