. docs/evidence/verb-tree/toolchain-env.sh
cargo build --locked --features continuation-trials --bin cli --bin cli-trial-receiver --example continuation-trial
trial_root=$(mktemp -d "$PWD/target/continuation-XXXXXX")
target/debug/examples/continuation-trial prepare "$PWD/target/debug/cli" "$PWD/target/debug/cli-trial-receiver" "$trial_root/rehearsal"
target/debug/examples/continuation-trial rehearse "$trial_root/rehearsal/package"
target/debug/examples/continuation-trial check "$trial_root/rehearsal/package" "$trial_root/rehearsal-evaluation"
target/debug/examples/continuation-trial prepare "$PWD/target/debug/cli" "$PWD/target/debug/cli-trial-receiver" "$trial_root/recipient"
