use serde_json::Value;
fn main() {
    let cases = [
        (r#"{"type":"integer","minimum":18446744073709551616}"#, "18446744073709551617", true),
        (r#"{"type":"integer","minimum":18446744073709551616}"#, "18446744073709551615", false),
        (r#"{"type":"number","multipleOf":0.1}"#, "0.3", true),
        (r#"{"const":1e400}"#, "10e399", true),
        (r##"{"$defs":{"port":{"type":"integer","minimum":1}},"properties":{"port":{"$ref":"#/$defs/port"}},"required":["port"]}"##, r#"{"port":0}"#, false),
    ];
    for (schema, instance, expected) in cases {
        let schema: Value = serde_json::from_str(schema).unwrap();
        let instance: Value = serde_json::from_str(instance).unwrap();
        let validator = jsonschema::options().with_draft(jsonschema::Draft::Draft202012).offline().build(&schema).unwrap();
        assert_eq!(validator.is_valid(&instance), expected);
        for error in validator.iter_errors(&instance) { println!("{} | {} | {}", error.instance_path(), error.schema_path(), error); }
    }
    for schema in [r#"{"type":"bogus"}"#,r#"{"$ref":"https://example.invalid/schema"}"#,r#"{"$ref":"file:///tmp/schema.json"}"#] {
        let schema: Value = serde_json::from_str(schema).unwrap();
        assert!(jsonschema::options().with_draft(jsonschema::Draft::Draft202012).offline().build(&schema).is_err());
    }
    println!("PASS: exact numeric bounds, decimal divisibility, equivalent exponents, local reference paths, invalid schema, offline HTTP and file references");
}
