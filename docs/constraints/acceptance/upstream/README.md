# Retained JSON Schema cases

These files were fetched unchanged from the [JSON Schema Test Suite](https://github.com/json-schema-org/JSON-Schema-Test-Suite/tree/main/tests/draft2020-12) before application implementation for CLI-004. Each basename maps directly to that directory: `type.json`, `required.json`, `minimum.json`, `multipleOf.json`, `const.json`, `properties.json`, `additionalProperties.json`, `prefixItems.json`, `allOf.json`, `anyOf.json`, and `oneOf.json`.

The [download hashes](../../../evidence/cli-004/upstream.sha256) identify the exact captured bytes. The upstream branch can change; these local files and hashes are the trial input. The [license](LICENSE) is retained with them. The test iterates every case in each selected file and compares its published `valid` value with the application constraint interpreter. No failing cases are filtered out. This selection does not establish complete Draft 2020-12 conformance.
