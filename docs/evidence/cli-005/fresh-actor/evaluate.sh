#!/usr/bin/env bash
set -euo pipefail
cd /home/apnex/taceng/cli
trial_evidence=docs/evidence/cli-005/fresh-actor
trial_package=target/continuation-tTfDo9/recipient/package
trial_evaluation=target/continuation-tTfDo9/fresh-actor-evaluation
test ! -e "$trial_evaluation"
test ! -e "$trial_evidence/evaluation.log"
date -u '+%Y-%m-%d %H:%M:%S UTC' > "$trial_evidence/evaluation-start.txt"
sha256sum --check docs/evidence/cli-005/handover-files.sha256 > "$trial_evidence/immutable-inputs-before.log"
sha256sum "$trial_package/work.session.json" "$trial_package/completed.definition.json" "$trial_package/completed.interface.json" > "$trial_evidence/submitted-files.sha256"
sha256sum --check docs/evidence/cli-005/release-binaries.sha256 > "$trial_evidence/release-binaries.log"
set +e
target/release/examples/continuation-trial check "$trial_package" "$trial_evaluation" > "$trial_evidence/evaluation.log" 2>&1
trial_exit=$?
set -e
printf '%s\n' "$trial_exit" > "$trial_evidence/evaluation.exit"
date -u '+%Y-%m-%d %H:%M:%S UTC' > "$trial_evidence/evaluation-end.txt"
sha256sum --check "$trial_evidence/submitted-files.sha256" > "$trial_evidence/submission-preservation.log"
sha256sum --check docs/evidence/cli-005/handover-files.sha256 > "$trial_evidence/immutable-inputs-after.log"
exit "$trial_exit"
