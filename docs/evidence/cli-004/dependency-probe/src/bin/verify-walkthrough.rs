use serde_json::Value;
use std::{env, fs, path::Path};
fn read(path: impl AsRef<Path>) -> Value { serde_json::from_slice(&fs::read(path).unwrap()).unwrap() }
fn main() {
    let root = env::current_dir().unwrap();
    let demo = fs::read_to_string(root.join("docs/evidence/cli-004/demo-directory.txt")).unwrap();
    let demo = Path::new(demo.trim());
    let expected_schema = read(root.join("docs/constraints/acceptance/service.schema.json"));
    let expected_instance = read(root.join("docs/constraints/acceptance/service.expected.json"));
    let expected_api = read(root.join("docs/constraints/acceptance/openapi.expected.json"));
    assert_eq!(read(demo.join("service.schema.json")), expected_schema);
    assert_eq!(read(demo.join("openapi.json")), expected_api);
    let instance = read(demo.join("instance.session.json"));
    assert_eq!(instance["candidate"], expected_instance);
    assert_eq!(instance["accepted"], expected_instance);
    assert_eq!(instance["active_constraint"]["schema"], expected_schema);
    assert_eq!(instance["constraint_mode"], "json-schema-2020-12");
    let api = read(demo.join("openapi.session.json"));
    assert_eq!(api["candidate"], expected_api);
    assert!(api.get("active_constraint").is_none());
    assert!(api.get("active_interface").is_none());
    let mut responses = 0;
    let mut expected_rejections = 0;
    for file in ["schema-construction.jsonl", "schema-save.jsonl", "instance-attachment.jsonl", "instance-authoring.jsonl", "openapi-construction.jsonl", "openapi-save.jsonl"] {
        let text = fs::read_to_string(demo.join(file)).unwrap();
        let mut previous_revision: Option<Value> = None;
        for line in text.lines() {
            let event: Value = serde_json::from_str(line).unwrap();
            if event["event"] == "response" {
                responses += 1;
                if event["status"] != "ok" {
                    assert_eq!(file, "instance-authoring.jsonl");
                    assert_eq!(event["operation"], "commit");
                    assert_eq!(event["error"]["code"], "SCHEMA_VIOLATION");
                    assert_eq!(event["error"]["validation"]["findings"][0]["instance_path"], "/port");
                    assert_eq!(event["error"]["validation"]["findings"][0]["schema_path"], "/$defs/port/type");
                    assert_eq!(event["mutation"], "none");
                    assert_eq!(Some(&event["session"]["revision"]), previous_revision.as_ref());
                    expected_rejections += 1;
                }
            } else { assert_eq!(event["event"], "session_open"); }
            previous_revision = Some(event["session"]["revision"].clone());
        }
    }
    assert_eq!(expected_rejections, 1);
    println!("PASS: {responses} documented responses, exactly {expected_rejections} intentional rejected commit; schema, accepted instance, attachment, and OpenAPI output match independent expected documents");
}
