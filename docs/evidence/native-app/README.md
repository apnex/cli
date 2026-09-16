# Native integration evidence

`cli-checks.tsv` and the unprefixed CLI check captures are the final contributor gate.
`agp-native-gate.txt` is the final complete consumer gate, including five native tests and twelve original e2e tests.
`source.sha256` covers the CLI runtime/test inputs; `consumer-source.sha256` covers the AGP Rust entry point, data, scripts, and native tests.
The manifests do not claim to hash every document or hosted workflow in either repository.

`initial-*` captures retain discovery, intermediate failures, corrections, and operator observations.
An intermediate capture can be a snapshot while a command is still active; only the final gate files above establish gate completion.
The result record cites individual completed observations explicitly.
`agp-baseline.txt` preserves the unchanged preimplementation AGP test run.

`operator-guide.mjs` is the exact local evidence driver, with historical workspace paths.
The portable consumer workflow is documented in AGP's guide; the driver does not replace it.
No successful IPv6 transport, independent review, or production deployment is claimed.
