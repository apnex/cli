. docs/evidence/verb-tree/toolchain-env.sh
cargo build --locked --release --features continuation-trials --bin cli --bin cli-trial-receiver --example continuation-trial
trial_root=$(mktemp -d "$PWD/target/continuation-XXXXXX")
target/release/examples/continuation-trial prepare "$PWD/target/release/cli" "$PWD/target/release/cli-trial-receiver" "$trial_root/rehearsal"
target/release/examples/continuation-trial rehearse "$trial_root/rehearsal/package"
target/release/examples/continuation-trial check "$trial_root/rehearsal/package" "$trial_root/rehearsal-evaluation"
target/release/examples/continuation-trial prepare "$PWD/target/release/cli" "$PWD/target/release/cli-trial-receiver" "$trial_root/recipient"
