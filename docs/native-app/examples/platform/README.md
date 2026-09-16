# Run a local platform prototype

This example configures a native application without an HTTP provider or endpoint settings.\
It reuses the existing [platform definition recipe](../../../composition/examples/platform/platform.commands) and adds an [application profile recipe](application.commands).\
Both documents are constructed through typed authoring commands, without editing serialized JSON.\
The configured commands simulate local state; they make no infrastructure changes.

## Construct both documents

Run from the CLI repository with its Rust prerequisites available.\
The generated files and receipts stay in a new directory under `target/`.

Build and author the application:
```sh
cargo build --locked --bin cli
cli_project_root="$PWD"
cli_platform_dir="$(mktemp -d "$cli_project_root/target/platform-app-XXXXXX")"
(
  cd "$cli_platform_dir"
  "$cli_project_root/target/debug/cli" \
    --definition "$cli_project_root/docs/authoring/operations.json" \
    --compose --session definition-session.json --create \
    --intent-file "$cli_project_root/docs/composition/examples/platform/TASK.md" \
    --commands < "$cli_project_root/docs/composition/examples/platform/platform.commands" \
    > definition-authoring.jsonl
  "$cli_project_root/target/debug/cli" \
    --definition "$cli_project_root/docs/authoring/operations.json" \
    --session application-session.json --create \
    --intent-file "$cli_project_root/docs/composition/examples/platform/TASK.md" \
    --commands < "$cli_project_root/docs/native-app/examples/platform/application.commands" \
    > application-authoring.jsonl
)
```

---

## Use the configured interface

Inspect the application:
```sh
"$cli_project_root/target/debug/cli" app \
  "$cli_platform_dir/platform-definition.json" \
  "$cli_platform_dir/platform-application.json" help
"$cli_project_root/target/debug/cli" app \
  "$cli_platform_dir/platform-definition.json" \
  "$cli_platform_dir/platform-application.json" tree
```

Open its operator shell with a persistent session:
```sh
"$cli_project_root/target/debug/cli" app \
  "$cli_platform_dir/platform-definition.json" \
  "$cli_platform_dir/platform-application.json" \
  --session "$cli_platform_dir/run.json"
```

Enter these commands:
```text
services
inspect
scale 5
inspect
up
ls
exit
```

The service starts with two replicas; scaling changes the simulated value to five.\
Help and output retain simulation labels.\
Root listing, context navigation, and completion work without management configuration.\
Unbound operations such as `services restart` still reject invocation.

Verify the retained state in another invocation:
```sh
"$cli_project_root/target/debug/cli" app \
  "$cli_platform_dir/platform-definition.json" \
  "$cli_platform_dir/platform-application.json" \
  --session "$cli_platform_dir/run.json" services inspect
```

---

## Package or extend it

The [native application guide](../../USE.md#embed-the-same-documents) shows how to embed these same documents in an executable with a project-owned name.\
Adding a declared HTTP capability also requires a matching optional `http` profile; operator endpoint controls then become available.\
No example-specific dispatcher or renderer is required.
