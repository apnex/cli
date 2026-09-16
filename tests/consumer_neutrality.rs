//! Consumer names belong in examples and acceptance data, never in the shared runtime.

use std::fs;
use std::path::Path;

fn assert_consumer_neutral_sources(directory: &Path) {
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            assert_consumer_neutral_sources(&path);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let source = fs::read_to_string(&path).unwrap();
            let consumer_name = source
                .split(|character: char| !character.is_ascii_alphanumeric())
                .any(|word| {
                    word.to_ascii_lowercase().starts_with("agp")
                        || word.contains("AGP")
                        || word.contains("Agp")
                });
            assert!(
                !consumer_name,
                "Consumer-specific name in {}: keep consumer data and dispatch outside the runtime",
                path.display()
            );
        }
    }
}

#[test]
fn shared_runtime_contains_no_first_consumer_names() {
    assert_consumer_neutral_sources(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"));
}
