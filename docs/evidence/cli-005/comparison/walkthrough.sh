#!/usr/bin/env bash
set -euo pipefail
cd /home/apnex/taceng/cli
. docs/evidence/verb-tree/toolchain-env.sh
cargo build --locked --release --features continuation-trials --bin cli --bin cli-trial-receiver --example continuation-trial
comparison_root=$(mktemp -d "$PWD/target/authoring-comparison-XXXXXX")
printf '%s\n' "$comparison_root" > docs/evidence/cli-005/comparison/run-location.txt
target/release/examples/continuation-trial compare "$PWD/target/release/cli" "$PWD/target/release/cli-trial-receiver" "$comparison_root/run"
jq '.methods[] | {method, status, full_staged_continuation: .evaluation.full_staged_continuation, phases: .workflow.phases, external_work: .workflow.external_work}' "$comparison_root/run/comparison.json"
