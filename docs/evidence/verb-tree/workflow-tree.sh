printf 'discover\n' |
  "$cli_project_root/target/debug/cli" \
    --definition "$cli_project_root/docs/authoring/operations.json" \
    --compose --session "$cli_platform_dir/session.json" --commands |
  jq -ejr 'select(.operation == "discover" and .status == "ok") | .result.verb_tree_text'
