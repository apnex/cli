//! Configured native applications exercise real HTTP, persistent observations, and shared navigation.
mod authoring_fixture;
use authoring_fixture::AuthoringFixture;
use programmable_cli::cli_application::CliApplicationProfile;
use programmable_cli::cli_http_get::{validate_http_resource_path, validate_loopback_http_base};
use programmable_cli::cli_output_view::CliOutputView;
use programmable_cli::document_value::DocumentValue;
use serde_json::{Value, json};
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::Duration;

fn specification() -> Value {
    let command = |id| json!({"id":id,"help":"Read the catalog.","parameters":[],"binding":{"kind":"connected","provider":"json-http-get-v1","capability":"catalog.read"},"view":"items"});
    json!({"format":"cli-definition-v3","id":"catalog","description":"A configured catalog application.","mock_state":{},
        "contexts":{
            "root":{"parent":null,"related":[],"help":"Inspect catalogs.","commands":{"read":command("catalog.read")}},
            "catalog":{"parent":"root","related":[],"help":"Catalog records.","commands":{"show":command("catalog.show")}}
        },"views":{"items":{"help":"Catalog IDs.","require":{"op":"literal","value":true},"rows":{"op":"root","path":[{"key":"items"}]},
            "columns":[{"id":"id","heading":"ID","value":{"op":"row","path":[{"key":"id"}]},"format":{"kind":"plain"}}]}}})
}
fn profile() -> Value {
    json!({"format":"cli-application-v1","default_output":"table","context_help":true,
        "control_aliases":{"?":":help","help":":help","up":":up","top":":top","tree":":tree","exit":":exit"},
        "http":{"option":"--endpoint","environment":"CATALOG_TEST_URL","resources":{"catalog.read":{"path":"/v1/items","require":{"op":"eq","left":{"op":"root","path":[{"key":"kind"}]},"right":{"op":"literal","value":"Catalog"}}}}},
        "exit_codes":{"INVALID_HTTP_GRANT":2,"CAPABILITY_NOT_GRANTED":2,"HTTP_TRANSPORT_FAILED":4,"HTTP_STATUS_FAILED":5,"INVALID_HTTP_DOCUMENT":6,"OUTPUT_VIEW_FAILED":7}})
}
fn fixture() -> AuthoringFixture {
    let fixture = AuthoringFixture::new();
    fs::write(
        fixture.directory.join("spec.json"),
        specification().to_string(),
    )
    .unwrap();
    fs::write(fixture.directory.join("app.json"), profile().to_string()).unwrap();
    fixture
}
fn launch(fixture: &AuthoringFixture, args: &[&str], input: &str) -> Output {
    launch_environment(fixture, args, input, &[])
}
fn launch_environment(
    fixture: &AuthoringFixture,
    args: &[&str],
    input: &str,
    environment: &[(&str, &str)],
) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cli"))
        .current_dir(&fixture.directory)
        .env("PATH", "")
        .env("XDG_CONFIG_HOME", fixture.directory.join("config"))
        .env_remove("CATALOG_TEST_URL")
        .envs(environment.iter().copied())
        .env("HTTP_PROXY", "http://127.0.0.1:1")
        .env("ALL_PROXY", "http://127.0.0.1:1")
        .env("NO_PROXY", "")
        .args(["app", "spec.json", "app.json"])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn operator_fixture() -> AuthoringFixture {
    let fixture = fixture();
    let mut application = profile();
    application["operator"] =
        json!({"command_aliases":{"catalog.read":"catalog.show"},"context_listing":["ls","show"]});
    application["control_aliases"]["management"] = json!(":endpoint");
    application["control_aliases"]["/"] = json!(":top");
    fs::write(fixture.directory.join("app.json"), application.to_string()).unwrap();
    fixture
}

fn output_events(output: &Output) -> Vec<Value> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn operator_help_is_compact_while_structured_discovery_and_compatibility_commands_remain_complete()
{
    let fixture = operator_fixture();
    let result = launch(
        &fixture,
        &[],
        "?\n/\nls\nshow\ncatalog\n?\nup\ntree\nexit\n",
    );
    code(&result, 0);
    let text = String::from_utf8_lossy(&result.stdout);
    assert!(!text.contains("connected:json-http-get-v1"));
    assert!(!text.contains("granted:"));
    assert!(!text.contains("up=:up"));
    assert!(text.contains("management set <url>"));
    assert!(text.contains("Commands: show. Use help for details."));
    assert!(text.contains("`-- management\n"));
    assert!(text.contains("set <url> [--save]"));
    let help = launch(&fixture, &["--help"], "");
    code(&help, 0);
    assert!(help.stdout.len() < 800);
    let details = launch(&fixture, &["help", "--all"], "");
    code(&details, 0);
    assert!(String::from_utf8_lossy(&details.stdout).contains("read [connected:json-http-get-v1]"));
    let discovery = launch(&fixture, &["--events", "help"], "");
    code(&discovery, 0);
    let event = &output_events(&discovery)[0];
    assert_eq!(event["result"]["commands"][0]["word"], "read");
    assert_eq!(event["result"]["commands"][0]["alias_of"], "catalog.show");
    assert_eq!(event["result"]["context_help"], "Inspect catalogs.");
    assert_eq!(event["result"]["endpoint_control"]["word"], "management");
    assert!(
        event["result"]["endpoint_control"]["actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| action["word"] == "set" && action["parameters"][0]["name"] == "url")
    );
    assert_eq!(
        event["result"]["commands"][0]["requirement"]["granted"],
        false
    );
    let missing = launch(&fixture, &["catalog", "show"], "");
    code(&missing, 2);
    assert_eq!(
        String::from_utf8_lossy(&missing.stderr),
        "No management endpoint configured.\nUse management set <url>, then retry the command.\n"
    );
    let machine = launch(&fixture, &["--events", "read"], "");
    code(&machine, 2);
    let error: Value = serde_json::from_slice(&machine.stderr).unwrap();
    assert_eq!(error["error"]["code"], "CAPABILITY_NOT_GRANTED");
}

#[test]
fn operator_endpoint_switch_and_invalid_replacement_preserve_observations_without_exporting_authority()
 {
    let fixture = operator_fixture();
    let (first, first_server) = server(vec![(200, RESPONSE.to_vec()), (200, RESPONSE.to_vec())]);
    let (second, second_server) = server(vec![(
        200,
        br#"{"kind":"Catalog","items":[{"id":42}]}"#.to_vec(),
    )]);
    let result = launch(
        &fixture,
        &["--events"],
        &format!(
            "management set {first}\nread\nmanagement set http://localhost:80\nread\nmanagement set {second}\nread\nmanagement clear\nread\n:render items\n:export operator-export.json\nexit\n"
        ),
    );
    code(&result, 2);
    first_server.join().unwrap();
    second_server.join().unwrap();
    let events = output_events(&result);
    let reads: Vec<_> = events
        .iter()
        .filter(|event| event["operation"] == "invoke")
        .collect();
    assert_eq!(reads.len(), 3);
    assert!(
        reads[0]["result"]["invocation"]["output_json_text"]
            .as_str()
            .unwrap()
            .contains("9007199254740993")
    );
    assert_eq!(
        reads[0]["result"]["invocation"]["output_json_text"],
        reads[1]["result"]["invocation"]["output_json_text"]
    );
    assert!(
        reads[2]["result"]["invocation"]["output_json_text"]
            .as_str()
            .unwrap()
            .contains("42")
    );
    let historical = events
        .iter()
        .find(|event| event["result"]["historical"] == true)
        .unwrap();
    assert_eq!(
        historical["result"]["invocation"],
        reads[2]["result"]["invocation"]
    );
    let exported = fs::read_to_string(fixture.directory.join("operator-export.json")).unwrap();
    assert!(!exported.contains(&first));
    assert!(!exported.contains(&second));
    assert!(!fixture.directory.join("config").exists());
}

#[test]
fn saved_endpoint_reopens_with_explicit_option_then_environment_then_saved_precedence() {
    let fixture = operator_fixture();
    let saved = "http://127.0.0.1:47101";
    code(
        &launch(
            &fixture,
            &[],
            &format!("management set {saved}/\nmanagement save\nexit\n"),
        ),
        0,
    );
    let state = launch(&fixture, &["--events", "management", "show"], "");
    code(&state, 0);
    assert_eq!(output_events(&state)[0]["result"]["endpoint"], saved);
    assert_eq!(output_events(&state)[0]["result"]["source"], "saved");
    let environment = launch_environment(
        &fixture,
        &["--events", "management", "show"],
        "",
        &[("CATALOG_TEST_URL", "http://127.0.0.1:47102")],
    );
    code(&environment, 0);
    assert_eq!(
        output_events(&environment)[0]["result"]["endpoint"],
        "http://127.0.0.1:47102"
    );
    let explicit = launch_environment(
        &fixture,
        &[
            "--events",
            "--endpoint",
            "http://127.0.0.1:47103",
            "management",
            "show",
        ],
        "",
        &[("CATALOG_TEST_URL", "http://127.0.0.1:47102")],
    );
    code(&explicit, 0);
    assert_eq!(
        output_events(&explicit)[0]["result"]["endpoint"],
        "http://127.0.0.1:47103"
    );
    let restored = launch(
        &fixture,
        &["--events"],
        "management clear\nmanagement load\nmanagement show\n",
    );
    code(&restored, 0);
    assert_eq!(output_events(&restored)[2]["result"]["endpoint"], saved);
    code(
        &launch(&fixture, &[], "management clear\nmanagement save\n"),
        0,
    );
    let cleared = launch(&fixture, &["--events", "management"], "");
    code(&cleared, 0);
    assert!(output_events(&cleared)[0]["result"]["endpoint"].is_null());
    let separate = launch(
        &fixture,
        &["--config", "separate.json", "--events", "management"],
        "",
    );
    code(&separate, 0);
    assert!(output_events(&separate)[0]["result"]["endpoint"].is_null());
}

#[test]
fn one_shot_management_changes_require_explicit_persistence() {
    let fixture = operator_fixture();
    let url = "http://127.0.0.1:47104";
    code(&launch(&fixture, &["management", "set", url], ""), 2);
    assert!(!fixture.directory.join("config").exists());
    code(
        &launch(&fixture, &["management", "set", url, "--save"], ""),
        0,
    );
    let reopened = launch(&fixture, &["--events", "management"], "");
    code(&reopened, 0);
    assert_eq!(output_events(&reopened)[0]["result"]["endpoint"], url);
    code(&launch(&fixture, &["management", "clear", "--save"], ""), 0);
    let cleared = launch(&fixture, &["--events", "management"], "");
    code(&cleared, 0);
    assert!(output_events(&cleared)[0]["result"]["endpoint"].is_null());
}

#[test]
fn management_settings_reject_stale_saves_and_invalid_reload_without_destroying_a_selection() {
    use programmable_cli::cli_operator_settings::CliOperatorSettings;
    let fixture = operator_fixture();
    let profile = CliApplicationProfile::parse_application_profile(
        &fs::read(fixture.directory.join("app.json")).unwrap(),
    )
    .unwrap();
    let path = fixture.directory.join("settings.json");
    let open = || {
        CliOperatorSettings::open_operator_settings(
            "catalog",
            profile.operator.clone().unwrap(),
            profile.http.clone(),
            Some(path.clone()),
            None,
        )
        .unwrap()
    };
    let mut first = open();
    let mut second = open();
    first
        .select_operator_endpoint(Some("http://127.0.0.1:47101"))
        .unwrap();
    second
        .select_operator_endpoint(Some("http://127.0.0.1:47102"))
        .unwrap();
    first.save_operator_settings().unwrap();
    let bytes = fs::read(&path).unwrap();
    assert_eq!(
        second.save_operator_settings().unwrap_err().code,
        "MANAGEMENT_SETTINGS_CHANGED"
    );
    assert_eq!(fs::read(&path).unwrap(), bytes);
    second.reload_operator_settings().unwrap();
    assert_eq!(second.endpoint, first.endpoint);
    fs::write(&path, br#"{"format":"cli-management-v1","application":"other","endpoint":"http://127.0.0.1:47103"}"#).unwrap();
    assert!(second.reload_operator_settings().is_err());
    assert_eq!(second.endpoint, first.endpoint);
    assert!(second.save_operator_settings().is_err());
    assert!(fs::read_to_string(&path).unwrap().contains("other"));
}

#[test]
fn operator_aliases_reject_behavior_changes_unknown_targets_chains_and_listing_collisions() {
    let fixture = operator_fixture();
    let application: Value =
        serde_json::from_slice(&fs::read(fixture.directory.join("app.json")).unwrap()).unwrap();
    let mut definition = specification();
    definition["contexts"]["catalog"]["commands"]["show"]["view"] = Value::Null;
    let profile =
        CliApplicationProfile::parse_application_profile(application.to_string().as_bytes())
            .unwrap();
    assert!(
        profile
            .validate_application_definition(definition.to_string().as_bytes())
            .is_err()
    );
    for alias in [
        json!({"missing":"catalog.show"}),
        json!({"catalog.read":"catalog.read"}),
        json!({"catalog.read":"catalog.show","catalog.show":"catalog.read"}),
    ] {
        let mut invalid = application.clone();
        invalid["operator"]["command_aliases"] = alias;
        let profile =
            CliApplicationProfile::parse_application_profile(invalid.to_string().as_bytes())
                .unwrap();
        assert!(
            profile
                .validate_application_definition(specification().to_string().as_bytes())
                .is_err()
        );
    }
    let mut invalid = application;
    invalid["operator"]["context_listing"] = json!(["management"]);
    let profile =
        CliApplicationProfile::parse_application_profile(invalid.to_string().as_bytes()).unwrap();
    assert!(
        profile
            .validate_application_definition(specification().to_string().as_bytes())
            .is_err()
    );
}

#[test]
fn operator_completion_exposes_root_listing_navigation_and_management_actions() {
    use programmable_cli::authoring_protocol::SessionRevision;
    use programmable_cli::cli_definition::CliDefinition;
    use programmable_cli::cli_interface::{ActiveCliInterface, CliInterfaceOrigin};
    use programmable_cli::cli_run_completion::CliRunCompleter;
    use programmable_cli::cli_run_routes::CliRunRoutes;
    use reedline::Completer;
    let fixture = operator_fixture();
    let profile = CliApplicationProfile::parse_application_profile(
        &fs::read(fixture.directory.join("app.json")).unwrap(),
    )
    .unwrap();
    let definition =
        CliDefinition::parse_cli_definition(specification().to_string().as_bytes()).unwrap();
    let mut routes = CliRunRoutes::from_cli_definition(&definition).unwrap();
    routes
        .configure_operator_routes(&definition, profile.operator)
        .unwrap();
    routes
        .configure_control_aliases(&definition, profile.control_aliases)
        .unwrap();
    let active = ActiveCliInterface::initialize_cli_interface(
        definition,
        CliInterfaceOrigin {
            intent_text: "Operator completion.".into(),
            revision: SessionRevision(0),
        },
    );
    let mut completer = CliRunCompleter::new_run_completer(active, routes);
    for (line, wanted) in [
        ("", "ls"),
        ("", "/"),
        ("", "management"),
        ("management ", "set"),
        ("management ", "save"),
        ("management ", "load"),
    ] {
        assert!(
            completer
                .complete(line, line.len())
                .suggestions()
                .iter()
                .any(|item| item.value == wanted),
            "{line:?} -> {wanted}"
        );
    }
}

#[test]
fn operator_settings_reject_invalid_documents_and_application_directory_aliases() {
    let fixture = operator_fixture();
    let display = launch(
        &fixture,
        &["--config", "settings\tescaped.json", "management"],
        "",
    );
    code(&display, 0);
    assert!(String::from_utf8_lossy(&display.stdout).contains("settings\\tescaped.json"));
    assert!(!display.stdout.contains(&b'\t'));
    let path = fixture.directory.join("invalid-settings.json");
    for body in [
        b"not json".to_vec(),
        br#"{"format":"cli-management-v1","application":"other","endpoint":null}"#.to_vec(),
        br#"{"format":"cli-management-v1","application":"catalog","endpoint":"http://localhost:80"}"#.to_vec(),
        br#"{"format":"cli-management-v1","application":"catalog","endpoint":null,"unknown":true}"#.to_vec(),
        vec![b' '; 4097],
    ] {
        fs::write(&path, &body).unwrap();
        let rejected = launch(&fixture, &["--config", "invalid-settings.json", "management", "set", "http://127.0.0.1:47101", "--save"], "");
        assert!(!rejected.status.success());
        assert_eq!(fs::read(&path).unwrap(), body);
    }
    let profile = CliApplicationProfile::parse_application_profile(
        &fs::read(fixture.directory.join("app.json")).unwrap(),
    )
    .unwrap();
    for id in [".", ".."] {
        let mut definition = specification();
        definition["id"] = json!(id);
        assert!(
            profile
                .validate_application_definition(definition.to_string().as_bytes())
                .is_err()
        );
    }
    #[cfg(unix)]
    {
        let link = fixture.directory.join("linked-settings.json");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        let rejected = launch(
            &fixture,
            &["--config", "linked-settings.json", "management"],
            "",
        );
        assert!(!rejected.status.success());
        assert!(String::from_utf8_lossy(&rejected.stderr).contains("symlink"));
    }
}
fn code(output: &Output, expected: i32) {
    assert_eq!(
        output.status.code(),
        Some(expected),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
fn server(responses: Vec<(u16, Vec<u8>)>) -> (String, thread::JoinHandle<()>) {
    serve_listener(TcpListener::bind("127.0.0.1:0").unwrap(), responses)
}

fn serve_listener(
    listener: TcpListener,
    responses: Vec<(u16, Vec<u8>)>,
) -> (String, thread::JoinHandle<()>) {
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let thread = thread::spawn(move || {
        for (status, body) in responses {
            let deadline = std::time::Instant::now() + Duration::from_secs(15);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(
                            std::time::Instant::now() < deadline,
                            "HTTP request deadline"
                        );
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("{error}"),
                }
            };
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                let mut byte = [0];
                stream.read_exact(&mut byte).unwrap();
                request.push(byte[0]);
                assert!(request.len() < 16384);
            }
            let request = String::from_utf8(request).unwrap();
            assert!(
                request.starts_with("GET /v1/items HTTP/1.1\r\n"),
                "{request}"
            );
            assert!(
                request
                    .to_ascii_lowercase()
                    .contains("accept: application/json")
            );
            let header = format!(
                "HTTP/1.1 {status} Response\r\nContent-Length: {}\r\nLocation: http://127.0.0.1:1/elsewhere\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream
                .write_all(header.as_bytes())
                .and_then(|()| stream.write_all(&body));
        }
    });
    (url, thread)
}
const RESPONSE: &[u8] = br#"{"kind":"Catalog","items":[{"id":9007199254740993}]}"#;

#[test]
fn native_http_keeps_exact_json_and_tables_without_helpers_or_proxies() {
    let fixture = fixture();
    let (url, server) = server(vec![(200, RESPONSE.to_vec()), (200, RESPONSE.to_vec())]);
    let raw = launch(&fixture, &["read", "--json", "--endpoint", &url], "");
    code(&raw, 0);
    assert_eq!(
        String::from_utf8(raw.stdout).unwrap(),
        "{\"items\":[{\"id\":9007199254740993}],\"kind\":\"Catalog\"}\n"
    );
    let table = launch(&fixture, &["--endpoint", &url, "catalog", "show"], "");
    code(&table, 0);
    assert_eq!(
        String::from_utf8(table.stdout).unwrap(),
        "ID\n9007199254740993\n"
    );
    server.join().unwrap();
}

#[test]
fn http_errors_reject_status_redirect_invalid_json_and_contract_in_raw_mode() {
    let fixture = fixture();
    for (status, body, expected) in [
        (302, RESPONSE.to_vec(), 5),
        (503, RESPONSE.to_vec(), 5),
        (200, b"not json".to_vec(), 6),
        (200, b"{\"kind\":\"Wrong\"}".to_vec(), 6),
        (200, vec![b' '; 1024 * 1024 + 1], 6),
        (200, vec![0xff], 6),
    ] {
        let (url, server) = server(vec![(status, body)]);
        let output = launch(&fixture, &["read", "--json", "--endpoint", &url], "");
        code(&output, expected);
        assert!(output.stdout.is_empty());
        server.join().unwrap();
    }
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    code(&launch(&fixture, &["read", "--endpoint", &url], ""), 4);
}

#[test]
fn endpoint_and_profile_rejection_precede_any_invocation() {
    for url in [
        "http://localhost:80",
        "http://127.1:80",
        "https://127.0.0.1:80",
        "http://127.0.0.1:0",
        "http://127.0.0.1:65536",
        "http://127.0.0.1:80/a",
        "http://127.0.0.1:80?x",
        "http://127.0.0.1:80#x",
        "http://user@127.0.0.1:80",
        "http://127.0.0.1:80//",
    ] {
        assert!(validate_loopback_http_base(url).is_err(), "{url}");
    }
    assert_eq!(
        validate_loopback_http_base("http://[::1]:88/").unwrap(),
        "http://[::1]:88"
    );
    for path in [
        "//elsewhere",
        "/../secret",
        "/%2e%2e/secret",
        "/v1/x?other",
        "/v1/x#other",
        "/v1/x\r\nHeader:x",
    ] {
        assert!(validate_http_resource_path(path).is_err());
    }
    let fixture = fixture();
    code(&launch(&fixture, &["read"], ""), 2);
    code(&launch(&fixture, &["read", "--json", "--json"], ""), 2);
    code(
        &launch(&fixture, &["read", "--endpoint", "http://localhost:80"], ""),
        2,
    );
    let structured = launch(
        &fixture,
        &["read", "--events", "--endpoint", "http://localhost:80"],
        "",
    );
    code(&structured, 2);
    let event: Value = serde_json::from_slice(&structured.stderr).unwrap();
    assert_eq!(event["error"]["code"], "INVALID_HTTP_GRANT");
    let mut invalid = profile();
    invalid["control_aliases"]["read"] = json!(":help");
    let profile =
        CliApplicationProfile::parse_application_profile(invalid.to_string().as_bytes()).unwrap();
    assert!(
        profile
            .validate_application_definition(specification().to_string().as_bytes())
            .is_err()
    );
    invalid["http"]["option"] = json!("--json");
    assert!(
        CliApplicationProfile::parse_application_profile(invalid.to_string().as_bytes()).is_err()
    );
}

#[test]
fn literal_ipv6_grant_reads_or_reports_unavailable_transport() {
    let fixture = fixture();
    let listener = match TcpListener::bind("[::1]:0") {
        Ok(listener) => listener,
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::AddrNotAvailable | std::io::ErrorKind::Unsupported
            ) =>
        {
            let result = launch(
                &fixture,
                &["read", "--json", "--endpoint", "http://[::1]:9"],
                "",
            );
            code(&result, 4);
            assert!(result.stdout.is_empty());
            println!(
                "UNRUNNABLE: successful IPv6 read because loopback bind failed: {error}; unavailable-transport status verified instead."
            );
            return;
        }
        Err(error) => panic!("Unexpected IPv6 fixture failure: {error}"),
    };
    let (url, server) = serve_listener(listener, vec![(200, RESPONSE.to_vec())]);
    let result = launch(&fixture, &["read", "--json", "--endpoint", &url], "");
    code(&result, 0);
    server.join().unwrap();
}

#[test]
fn configured_control_completion_follows_the_same_context_routes() {
    use programmable_cli::authoring_protocol::SessionRevision;
    use programmable_cli::cli_definition::CliDefinition;
    use programmable_cli::cli_interface::{ActiveCliInterface, CliInterfaceOrigin};
    use programmable_cli::cli_run_completion::CliRunCompleter;
    use programmable_cli::cli_run_routes::CliRunRoutes;
    use reedline::Completer;
    let definition =
        CliDefinition::parse_cli_definition(specification().to_string().as_bytes()).unwrap();
    let profile =
        CliApplicationProfile::parse_application_profile(profile().to_string().as_bytes()).unwrap();
    let mut routes = CliRunRoutes::from_cli_definition(&definition).unwrap();
    routes
        .configure_control_aliases(&definition, profile.control_aliases)
        .unwrap();
    let mut active = ActiveCliInterface::initialize_cli_interface(
        definition,
        CliInterfaceOrigin {
            intent_text: "Completion check.".into(),
            revision: SessionRevision(0),
        },
    );
    let mut completer = CliRunCompleter::new_run_completer(active.clone(), routes);
    assert!(
        completer
            .complete("?", 1)
            .suggestions()
            .iter()
            .any(|item| item.value == "?")
    );
    assert!(
        completer
            .complete("help c", 6)
            .suggestions()
            .iter()
            .any(|item| item.value == "catalog")
    );
    active.enter_cli_context("catalog").unwrap();
    completer.refresh_run_completion(&active);
    assert!(
        completer
            .complete("s", 1)
            .suggestions()
            .iter()
            .any(|item| item.value == "show")
    );
}

#[test]
fn navigation_shortcuts_help_and_tree_share_configured_routes() {
    let fixture = fixture();
    let result = launch(&fixture, &[], "catalog\n?\nup\ntop\ntree\nexit\n");
    code(&result, 0);
    let output = String::from_utf8(result.stdout).unwrap();
    assert!(output.contains("catalog / catalog\nCatalog records."));
    assert!(output.contains("show [connected:json-http-get-v1]"));
    assert!(output.contains("up=:up"));
    assert!(output.contains("`-- catalog/"));
    code(&launch(&fixture, &["help", "catalog", "show"], ""), 0);
}

#[test]
fn http_history_survives_failed_read_export_and_reopen_without_authority() {
    let fixture = fixture();
    let (url, server) = server(vec![(200, RESPONSE.to_vec()), (503, Vec::new())]);
    let args = ["--endpoint", &url, "--session", "run.json", "--events"];
    let result = launch(
        &fixture,
        &args,
        "read\nread\n:render items\n:export portable.json\n",
    );
    code(&result, 5);
    server.join().unwrap();
    let events: Vec<Value> = String::from_utf8(result.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let receipt = &events[0]["result"]["invocation"];
    assert_eq!(receipt["observation"]["provider"], "json-http-get-v1");
    assert_eq!(events[1]["result"]["invocation"], *receipt);
    let checkpoint: Value =
        serde_json::from_slice(&fs::read(fixture.directory.join("run.json")).unwrap()).unwrap();
    let stored = checkpoint.to_string();
    assert!(!stored.contains(&url));
    let resumed = launch(
        &fixture,
        &["--session", "run.json", "--events"],
        ":status\nread\n",
    );
    code(&resumed, 2);
    assert!(
        String::from_utf8(resumed.stdout)
            .unwrap()
            .contains("json-http-get-v1")
    );
    fs::copy(
        fixture.directory.join("portable.json"),
        fixture.directory.join("spec.json"),
    )
    .unwrap();
    let imported = launch(&fixture, &["--events"], ":render items\nread\n");
    code(&imported, 2);
    assert!(
        String::from_utf8(imported.stdout)
            .unwrap()
            .contains("9007199254740993")
    );
}

#[test]
fn renderer_failure_preserves_successful_http_receipt_and_application_status() {
    let fixture = fixture();
    let (url, server) = server(vec![(200, br#"{"kind":"Catalog","items":false}"#.to_vec())]);
    let result = launch(&fixture, &["--endpoint", &url, "--events", "read"], "");
    code(&result, 7);
    let response: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(response["status"], "ok");
    assert_eq!(response["mutation"], "applied");
    assert_eq!(response["result"]["presentation"]["status"], "error");
    assert_eq!(response["result"]["invocation"]["effect"], "external_read");
    server.join().unwrap();
}

#[test]
fn object_entries_and_collection_lengths_preserve_keys_values_and_order() {
    let view: CliOutputView = serde_json::from_value(json!({"help":"Map values.","require":{"op":"literal","value":true},"rows":{"op":"entries","input":{"op":"root","path":[]}},"columns":[
        {"id":"key","heading":"KEY","value":{"op":"row","path":[{"key":"key"}]},"format":{"kind":"plain"}},
        {"id":"count","heading":"COUNT","value":{"op":"length","input":{"op":"row","path":[{"key":"value"}]}},"format":{"kind":"plain"}}
    ]})).unwrap();
    let input =
        DocumentValue::parse_document(r#"{"z":[1,2],"0":{"n":9007199254740993},"a":[]}"#).unwrap();
    let output = view.present_output_document("map", &input).unwrap();
    assert_eq!(
        output.rows_json_text,
        r#"[{"count":1,"key":"0"},{"count":0,"key":"a"},{"count":2,"key":"z"}]"#
    );
    for input in ["[]", "null", "{\"x\":true}"] {
        assert!(
            view.present_output_document("map", &DocumentValue::parse_document(input).unwrap())
                .is_err()
        );
    }
    let mut malformed = serde_json::to_value(&view).unwrap();
    malformed["rows"]["extra"] = json!(true);
    let malformed: CliOutputView = serde_json::from_value(malformed).unwrap();
    assert!(malformed.validate_output_view().is_err());
}
