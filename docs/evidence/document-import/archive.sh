#!/usr/bin/env bash
set -euo pipefail
cd /home/apnex/taceng/cli
import_evidence=docs/evidence/document-import
import_walkthrough=$(cat "$import_evidence/walkthrough-location.txt")
import_comparison=$(cat "$import_evidence/comparison-location.txt")/run
for import_origin in "$import_walkthrough" "$import_comparison"; do
    if [ "$import_origin" = "$import_walkthrough" ]; then
        import_archive="$import_evidence/walkthrough"
    else
        import_archive="$import_evidence/comparison"
    fi
    test ! -e "$import_archive"
    mkdir "$import_archive"
    rg --files --hidden "$import_origin" | sort > "$import_archive/source-files.txt"
    while IFS= read -r import_file; do
        sha256sum "$import_file"
    done < "$import_archive/source-files.txt" > "$import_archive/source-inventory.sha256"
    while IFS= read -r import_file; do
        import_relative=${import_file#"$import_origin/"}
        case "$import_relative" in
            */cli|*/receiver) continue ;;
            *.md) import_relative=${import_relative%.md}.txt ;;
        esac
        import_copy="$import_archive/$import_relative"
        test ! -e "$import_copy"
        mkdir -p "$(dirname "$import_copy")"
        cp "$import_file" "$import_copy"
        cmp "$import_file" "$import_copy"
        printf '%s\t%s\n' "$import_file" "$import_copy"
    done < "$import_archive/source-files.txt" > "$import_archive/archive-map.tsv"
done
rg --files --hidden "$import_evidence/walkthrough" "$import_evidence/comparison" | sort > "$import_evidence/archive-files.txt"
while IFS= read -r import_file; do
    sha256sum "$import_file"
done < "$import_evidence/archive-files.txt" > "$import_evidence/artifact-inventory.sha256"
sha256sum -c "$import_evidence/artifact-inventory.sha256" > "$import_evidence/archive-integrity.log"
sha256sum Cargo.toml Cargo.lock src/*.rs tests/*.rs tools/continuation/*.rs tools/scaffold/tests/project_records.rs docs/authoring/operations.json docs/authoring/acceptance/import-cases.json > "$import_evidence/source.sha256"
sha256sum target/release/cli target/release/cli-trial-receiver target/release/examples/continuation-trial > "$import_evidence/release-binaries.sha256"
printf 'Archived walkthrough and comparison text with byte-preservation checks.\n'
