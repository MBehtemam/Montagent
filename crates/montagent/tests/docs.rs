//! The published docs reach an agent on both surfaces: `resources/read` carries the cache
//! fields a validating client requires, and `montagent docs` prints the same bytes.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use serde_json::{Value, json};

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_montagent")
}

/// One `resources/read` over stdio, speaking `version`, and its response.
fn read_resource(version: &str, uri: &str) -> Value {
    let mut child = Command::new(binary())
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn the MCP server");
    let mut stdin = child.stdin.take().unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let mut ask = |message: Value| writeln!(stdin, "{message}").and_then(|_| stdin.flush());
    ask(
        json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{
        "protocolVersion":version,"capabilities":{},
        "clientInfo":{"name":"docs-test","version":"0"}}}),
    )
    .unwrap();
    lines.next().unwrap().unwrap();
    ask(json!({"jsonrpc":"2.0","method":"notifications/initialized"})).unwrap();
    ask(json!({"jsonrpc":"2.0","id":1,"method":"resources/read","params":{"uri":uri}})).unwrap();
    let response = loop {
        let value: Value = serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();
        if value["id"] == json!(1) {
            break value;
        }
    };
    drop(stdin);
    let _ = child.wait();
    response
}

#[test]
fn a_resource_read_carries_ttl_ms_and_cache_scope() {
    let response = read_resource("2025-06-18", "montagent://format.md");
    let result = &response["result"];
    assert!(result["contents"][0]["text"].is_string(), "{response}");
    assert!(result["ttlMs"].is_u64(), "ttlMs missing: {response}");
    assert_eq!(result["cacheScope"], "public", "{response}");
}

#[test]
fn docs_prints_the_bytes_resources_read_serves() {
    let out = Command::new(binary())
        .args(["docs", "format"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let served = montagent_core::resources::read("montagent://format.md").unwrap();
    assert_eq!(String::from_utf8(out.stdout).unwrap(), served);
}

#[test]
fn docs_with_no_name_lists_what_can_be_named_and_an_unknown_name_is_refused() {
    let list = Command::new(binary()).arg("docs").output().unwrap();
    assert!(list.status.success());
    let list = String::from_utf8(list.stdout).unwrap();
    assert!(list.contains("montagent://format.md") && list.contains("montagent://format/text.md"));

    let bad = Command::new(binary())
        .args(["docs", "nonsense"])
        .output()
        .unwrap();
    assert_eq!(bad.status.code(), Some(3));
    assert!(bad.stdout.is_empty());
}
