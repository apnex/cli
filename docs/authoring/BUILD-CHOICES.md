# First authoring slice: build choices

**Status: selected engineering approach for the bounded CLI-002 implementation.**
These choices support the [session contract](SESSION-CONTRACT.md); they do not ratify a permanent public API or claim an implemented runtime.
The subsequent [CLI-002 record](CLI-002.md) identifies the actual implementation and application evidence separately from the dependency probe retained here.

## Packaging

Use Rust and one unpublished Cargo package for the first application slice.
Keep document values, authoring transitions, runtime dispatch, persistence, and interaction in named internal modules under their owning responsibility scopes.
The entry point only assembles them.
The seven responsibility records live under `docs/layers/`; they are documentation scopes, not source module locations.
Do not convert them into seven independently versioned crates without concrete consumers.

The terminal and machine transports call the same request dispatcher.
The [bootstrap operation declaration](operations.json) is loaded as data and binds registered handler and argument-codec contracts.
No application-specific behavior or network client is needed for this journey.

---

## Dependencies and their consumers

| Dependency | Consumer and purpose | Boundary |
|---|---|---|
| `serde` | Structured request, declaration, and checkpoint codecs. | Reject unknown fields and duplicate decoded object keys explicitly. |
| `serde_json` with `raw_value` | JSON token validation and lossless number storage. | Do not deserialize authored numbers through floating-point values. |
| `sha2` | Definition identity and export byte receipts. | A digest identifies observed bytes; it does not certify intent or external behavior. |
| `uuid` | New opaque session identities. | Generated identifiers are excluded from cross-session semantic comparisons. |
| `reedline` | Interactive line editing and runtime-driven completion. | It presents and compiles input; it does not own document mutation. |
| Rust standard filesystem APIs | Cooperative locking and local publication primitives. | Filesystem failure behavior still requires application-level fault injection. |

The exact dependency resolution used in the probe is retained in its [Cargo lockfile](../evidence/cli-001/dependency-probe/Cargo.lock).
The application now carries its own [Cargo lockfile](../../Cargo.lock), distinct from the historical probe resolution.

The proposed internal document enum separates object, array, string, number token, boolean, and null.
Objects use an ordered map and arrays use an ordered vector.
Number tokens use validated `RawValue` storage, whose [documented serialization behavior](https://docs.rs/serde_json/latest/serde_json/value/struct.RawValue.html) preserves the underlying JSON text.
String decoding and recursive duplicate-key rejection remain separate obligations; accepting a raw fragment alone is insufficient.

---

## Probe evidence

The [retained dependency probe](../evidence/cli-001/dependency-probe/src/main.rs) tests actual library and filesystem behavior.
Its observations are recorded in the [execution evidence](../evidence/cli-001/verification.json).

| Observed in the probe | What remains unproven |
|---|---|
| Large integers, large exponents, negative zero, and fractional zeros retain exact number tokens. | Recursive document mutation and complete checkpoint fidelity. |
| Invalid numeric tokens and unpaired surrogate string decoding are rejected. | Every malformed request and nested checkpoint path. |
| A custom visitor rejects decoded duplicate object keys. | Recursive enforcement in the future document codec. |
| A second file handle cannot acquire the held local exclusive lock. | Competing application processes and crash recovery of the complete session. |
| Local replacement, file and directory synchronization, and create-only publication calls succeed. | Power-loss durability and every injected publication failure boundary. |
| The selected editor can be constructed with the resolved dependencies. | Interactive completion, rendering, and terminal-machine parity. |

The original environment had no `cargo`, `rustc`, or `rustup` on its path.
The probe used a checksum-verified official installer and an isolated temporary toolchain without changing shell profiles or installing the application.
The [Rust installation documentation](https://doc.rust-lang.org/book/ch01-01-installation.html) describes normal toolchain setup for a future implementation environment.

---

## Reproduce the dependency probe

Prerequisites are a Rust compiler and Cargo on the path, a C linker, and the local Linux filesystem capabilities named in the session contract.
Run from the repository root; dependency retrieval may need network access if the lockfile's packages are not cached.
The probe creates and removes its own temporary filesystem fixtures.

Run the retained probe:
```sh
cargo run --locked --manifest-path docs/evidence/cli-001/dependency-probe/Cargo.toml
```

This command executes dependency assertions, not the application acceptance journey.
The probe's build output stays in its ignored `target` directory unless the caller supplies a separate Cargo target directory.

---

## Mechanics, rationale, and consequence

### Mechanics

Choose dependencies against concrete contracts, retain exact resolution and probe source, and carry untested behavior into the application acceptance corpus.

### Rationale

The original conversation expressed a Rust preference, but compiler safety, tiny binaries, and universal superiority are not evidence supplied by that preference.
The selected package must earn its claims through the bounded authoring workflow.

### Consequence of violation

A successful library probe can be mistaken for a verified editor or recovery engine unless its actual scope and remaining obligations travel with it.
