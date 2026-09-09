//! Three separately compiled startup probes alter actual registrations or declared argument data.

use programmable_cli::authoring_operations::registered_authoring_handlers;
use programmable_cli::authoring_runtime::AuthoringRuntime;
use programmable_cli::operation_definition::OperationDefinition;
use programmable_cli::storage_faults::StorageFaultControl;
use serde_json::Value;
use std::path::Path;

pub fn run_handler_contract_probe(change: &str) {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(
        arguments.len(),
        2,
        "Probe requires declaration and proposed checkpoint paths"
    );
    let mut bytes = std::fs::read(&arguments[0]).unwrap();
    let mut registrations = registered_authoring_handlers();
    match change {
        "missing" => {
            assert!(
                registrations.remove("authoring.delete").is_some(),
                "INVALID: missing-handler mutation did not land"
            );
        }
        "extra" => {
            assert!(
                registrations
                    .insert(
                        "authoring.unadvertised".into(),
                        registrations["authoring.top"].clone()
                    )
                    .is_none(),
                "INVALID: extra handler already existed"
            );
        }
        "argument" => {
            let mut declaration: Value = serde_json::from_slice(&bytes).unwrap();
            let operation = declaration["operations"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|operation| operation["handler"] == "authoring.set")
                .unwrap();
            assert_eq!(
                operation["arguments"][0]["name"], "path",
                "INVALID: argument mutation did not address original field"
            );
            operation["arguments"][0]["name"] = Value::String("location".into());
            bytes = serde_json::to_vec(&declaration).unwrap();
            assert_ne!(bytes, std::fs::read(&arguments[0]).unwrap());
        }
        _ => unreachable!(),
    }
    eprintln!("LANDED {change}");
    let outcome =
        OperationDefinition::from_definition_bytes(&bytes, registrations).and_then(|definition| {
            AuthoringRuntime::open_authoring_session(
                Path::new(&arguments[1]),
                definition,
                Some("Contract probe".into()),
                StorageFaultControl::default(),
            )
        });
    match outcome {
        Err(error) => {
            println!("{}", error.code);
            std::process::exit(1);
        }
        Ok(_) => {
            println!("ACCEPTED_INCOMPATIBLE_DEFINITION");
            std::process::exit(0);
        }
    }
}
