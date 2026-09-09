#!/usr/bin/env bash
set -euo pipefail
cd /home/apnex/taceng/cli
comparison_archive=docs/evidence/cli-005/comparison
comparison_run=$(cat "$comparison_archive/run-location.txt")/run
test -d "$comparison_run"
test ! -e "$comparison_archive/walkthrough"
mkdir "$comparison_archive/walkthrough"
rg --files --hidden "$comparison_run" | sort > "$comparison_archive/source-files.txt"
while IFS= read -r comparison_file; do
    sha256sum "$comparison_file"
done < "$comparison_archive/source-files.txt" > "$comparison_archive/source-inventory.sha256"
while IFS= read -r comparison_file; do
    comparison_relative=${comparison_file#"$comparison_run/"}
    case "$comparison_relative" in
        */cli|*/receiver) continue ;;
        *.md) comparison_relative=${comparison_relative%.md}.txt ;;
    esac
    comparison_copy="$comparison_archive/walkthrough/$comparison_relative"
    test ! -e "$comparison_copy"
    mkdir -p "$(dirname "$comparison_copy")"
    cp "$comparison_file" "$comparison_copy"
    cmp "$comparison_file" "$comparison_copy"
    printf '%s\t%s\n' "$comparison_file" "$comparison_copy"
done < "$comparison_archive/source-files.txt" > "$comparison_archive/archive-map.tsv"
rg --files "$comparison_archive/walkthrough" | sort > "$comparison_archive/archive-files.txt"
while IFS= read -r comparison_file; do
    sha256sum "$comparison_file"
done < "$comparison_archive/archive-files.txt" > "$comparison_archive/artifact-inventory.sha256"
for comparison_document in README.md docs/ARCHITECTURE.md docs/resume-work.md docs/reuse/CLI-005.md; do
    comparison_copy="$comparison_archive/before/${comparison_document//\//_}.txt"
    test ! -e "$comparison_copy"
    cp "$comparison_document" "$comparison_copy"
done
rg '  (src/|Cargo\.(toml|lock)$|docs/(authoring|composition|constraints)/operations\.json$)' docs/evidence/cli-005/source.sha256 > "$comparison_archive/unchanged-runtime.sha256"
sha256sum -c "$comparison_archive/unchanged-runtime.sha256" > "$comparison_archive/runtime-continuity.log"
sha256sum Cargo.toml Cargo.lock src/*.rs tools/continuation/*.rs tests/continuation_trials.rs docs/reuse/acceptance/TASK.md docs/reuse/acceptance/*.commands docs/reuse/acceptance/*.json docs/reuse/comparison/* docs/authoring/operations.json docs/composition/operations.json docs/constraints/operations.json > "$comparison_archive/source.sha256"
sha256sum target/release/cli target/release/cli-trial-receiver target/release/examples/continuation-trial > "$comparison_archive/release-binaries.sha256"
sha256sum -c "$comparison_archive/artifact-inventory.sha256" > "$comparison_archive/archive-integrity.log"
printf 'Archived text evidence and verified byte preservation.\n'
