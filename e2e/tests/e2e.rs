//! Drive the packed component through `act run --mcp` with a real MCP client.
//!
//! This replaces the python fastmcp/pytest suite that still sits alongside it
//! in this directory: the tests observe exactly what an agent observes, over
//! the same client stack (`rmcp`) the host bridge itself is built on.
//!
//! Env: WASM — path to the packed component (default: the component's
//!      release build output);
//!      ACT  — the act invocation (default `act`; `npx @actcore/act`, the
//!             component justfile's default, also works — whitespace-split,
//!             like the shlex.split the python conftest did).

use std::path::PathBuf;

use rmcp::{
    ServiceExt,
    model::CallToolRequestParams,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use serde_json::{Value, json};

/// `().serve(transport)` hands back the client-role service running over the
/// child process: role first, the unit client handler second.
type Client = rmcp::service::RunningService<rmcp::service::RoleClient, ()>;

fn wasm_path() -> PathBuf {
    PathBuf::from(std::env::var("WASM").unwrap_or_else(|_| {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../target/wasm32-wasip2/release/component_encoding.wasm"
        )
        .into()
    }))
}

/// The ACT invocation, honouring the same override the component justfile
/// uses. Its default there is `npx @actcore/act` — two words — which cannot
/// be `argv[0]` for a non-shell spawn, so the value is whitespace-split into
/// program + leading args. Quoted paths with spaces are not a form this
/// fleet passes through `ACT`; a full shlex is deliberately not pulled in.
fn act_argv() -> Vec<String> {
    std::env::var("ACT")
        .unwrap_or_else(|_| "act".into())
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

/// Spawn `act run <wasm> --mcp`. No grant flags: `encoding` declares no
/// capability ceiling (act.toml carries only the component name), so there
/// is nothing to grant and nothing to refuse.
fn act_command() -> tokio::process::Command {
    let argv = act_argv();
    let mut cmd = tokio::process::Command::new(&argv[0]);
    cmd.args(&argv[1..]);
    cmd.arg("run").arg(wasm_path()).arg("--mcp");
    cmd
}

async fn connect() -> Client {
    let transport = TokioChildProcess::new(act_command()).expect("spawn act run --mcp");
    ().serve(transport)
        .await
        .expect("rmcp handshake with act run --mcp")
}

fn first_text_block(result: &rmcp::model::CallToolResult) -> &rmcp::model::TextContent {
    match result.content.first() {
        Some(rmcp::model::ContentBlock::Text(t)) => t,
        other => panic!("expected the first content block to be Text, got: {other:?}"),
    }
}

/// Call a tool and return its first text block's text. One `act` process per
/// call site — the python conftest kept the client function-scoped on
/// purpose (a component keeps state across calls within a host process, and
/// a fresh process is the safe default even for a stateless one), and the
/// translation keeps that shape.
async fn call_text(client: &Client, tool: &'static str, arguments: Value) -> String {
    let args = arguments.as_object().expect("arguments are an object").clone();
    let result = client
        .call_tool(CallToolRequestParams::new(tool).with_arguments(args))
        .await
        .expect("call_tool");
    assert_ne!(result.is_error, Some(true), "{tool} failed: {result:?}");
    first_text_block(&result).text.clone()
}

/// Assert a call fails with a specific ACT error kind, returning it.
///
/// The python conftest's `expect_error` fixture handled both arrival paths:
/// a JSON-RPC error response (`ErrorData.data`) for failures that are not
/// the guest's tool body (`list-tools`, sessions, traps), and an isError
/// result with the kind in `_meta` for guest `tool-event::error`s — the
/// path a tool test takes, because `call-tool` has no `result<>` wrapper.
/// Both are handled here so callers need not care.
async fn expect_error_kind(client: &Client, tool: &'static str, arguments: Value) -> String {
    let args = arguments.as_object().expect("arguments are an object").clone();
    let params = CallToolRequestParams::new(tool).with_arguments(args);
    match client.call_tool(params).await {
        Err(rmcp::ServiceError::McpError(e)) => e
            .data
            .as_ref()
            .and_then(|d| d.get("dev.actcore/error-kind"))
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| {
                panic!("expected dev.actcore/error-kind on the JSON-RPC error path, got {e:?}")
            }),
        Ok(result) => {
            assert_eq!(result.is_error, Some(true), "expected {tool} to fail: {result:?}");
            result
                .meta
                .as_ref()
                .and_then(|m| m.0.get("dev.actcore/error-kind"))
                .and_then(|v| v.as_str())
                .map(str::to_string)
                .unwrap_or_else(|| {
                    panic!("expected dev.actcore/error-kind in result _meta: {result:?}")
                })
        }
        Err(other) => panic!("unexpected transport failure: {other:?}"),
    }
}

/// The manifest probe from the python test_info.py: the packed artifact must
/// declare its name and a version. Also the fast-fail the python `wasm_path`
/// fixture provided — an unpacked wasm (raw `cargo build` output, no
/// `act:component` section) declares no ceiling, every grant is refused as
/// "outside ceiling", and the failures point anywhere but at the missing
/// metadata. The justfile's `test: build` ordering exists so this test finds
/// a packed artifact.
#[test]
fn manifest_reports_name_and_version() {
    let output = {
        let argv = act_argv();
        let mut cmd = std::process::Command::new(&argv[0]);
        cmd.args(&argv[1..]);
        cmd.args(["inspect", "component-manifest"])
            .arg(wasm_path())
            .output()
            .expect("run act inspect component-manifest")
    };
    assert!(
        output.status.success(),
        "inspect failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let manifest: Value = serde_json::from_slice(&output.stdout).expect("manifest is JSON");
    assert_eq!(
        manifest["std"]["name"], "encoding",
        "packed manifest must carry the component name"
    );
    assert!(
        manifest["std"]["version"].is_string(),
        "packed manifest must carry a version, got: {}",
        manifest["std"]["version"]
    );
}

/// The python test_tools.py: the component must expose its tools.
#[tokio::test]
async fn component_exposes_its_tools() {
    let client = connect().await;
    let tools = client.list_all_tools().await.expect("list_all_tools");
    assert!(
        tools.len() >= 1,
        "component must expose at least one tool, got: {:?}",
        tools.iter().map(|t| t.name.to_string()).collect::<Vec<_>>()
    );
    client.cancel().await.ok();
}

// Byte-oriented codecs against one canonical input. The python suite
// parametrized these ten cases (ported from the sixteen near-identical
// blocks in the old encode.hurl); the loop is where the file stays small.
/// (format, expected) — `None` is the schema default, argument omitted.
const ENCODE_BYTE_CASES: &[(Option<&str>, &str)] = &[
    (None, "aGVsbG8gd29ybGQ="),  // default (base64)
    (Some("base16"), "68656c6c6f20776f726c64"),
    (Some("base32"), "NBSWY3DPEB3W64TMMQ======"),
    (Some("base58"), "StV1DL6CwTryKyV"),
    (Some("base64-nopad"), "aGVsbG8gd29ybGQ"),
    (Some("base64url-nopad"), "aGVsbG8gd29ybGQ"),
    (Some("base32hex"), "D1IMOR3F41RMUSJCCG======"),
    (Some("base36"), "fuvrsivvnfrbjwajo"),
    (Some("base62"), "AAwf93rvy4aWQVw"),
    (Some("ascii85"), "<~BOu!rD]j7BEbo7~>"),
];

#[tokio::test]
async fn encode_hello_world_across_byte_codecs() {
    let client = connect().await;
    for (format, expected) in ENCODE_BYTE_CASES {
        // `None` means the argument is omitted, exercising the default.
        let args = match format {
            Some(f) => json!({"input": "hello world", "format": f}),
            None => json!({"input": "hello world"}),
        };
        let text = call_text(&client, "encode", args).await;
        assert_eq!(
            &text, expected,
            "encode with format {format:?} must yield exactly {expected:?}"
        );
    }
    client.cancel().await.ok();
}

// Text codecs (url/html/punycode) operate on the string itself, so they each
// need their own input rather than sharing "hello world".
#[tokio::test]
async fn encode_text_codecs() {
    let client = connect().await;
    for (text, format, expected) in [
        ("a b&c=d", "url", "a%20b%26c%3Dd"),
        ("x<y>&z", "html", "x&lt;y&gt;&amp;z"),
        ("münchen", "punycode", "mnchen-3ya"),
    ] {
        let args = json!({"input": text, "format": format});
        let out = call_text(&client, "encode", args).await;
        assert_eq!(
            out, expected,
            "encode with format {format:?} must yield exactly {expected:?}"
        );
    }
    client.cancel().await.ok();
}

/// `format` is a schema enum; an unrecognized value fails to deserialize
/// before `encode`'s body ever runs, which surfaces as std:invalid-args —
/// same kind as an explicit ActError::invalid_args call (measured via
/// act-sdk-macros' generated argument-deserialization arm).
#[tokio::test]
async fn encode_rejects_unknown_format() {
    let client = connect().await;
    let kind = expect_error_kind(
        &client,
        "encode",
        json!({"input": "hello", "format": "rot13"}),
    )
    .await;
    assert_eq!(kind, "std:invalid-args");
    client.cancel().await.ok();
}

/// A {"$bytes": "<base64>"} object is the transport's byte-string
/// projection: encode should treat it as raw bytes, not literal text.
#[tokio::test]
async fn encode_binary_input_via_bytes_envelope() {
    let client = connect().await;
    let text = call_text(
        &client,
        "encode",
        json!({"input": {"$bytes": "//79"}, "format": "base16"}),
    )
    .await;
    assert_eq!(text, "fffefd");
    client.cancel().await.ok();
}

/// A bare string is literal text to encode, not base64-decoded first.
#[tokio::test]
async fn encode_plain_string_input_is_literal_text() {
    let client = connect().await;
    let text = call_text(&client, "encode", json!({"input": "hi", "format": "base16"})).await;
    assert_eq!(text, "6869");
    client.cancel().await.ok();
}

// Byte-oriented codecs decoding back to "hello world" — the decode mirror
// of ENCODE_BYTE_CASES, with base36's input upper-case to pin the
// case-insensitivity of that decoder.
/// (format, encoded) — `None` is the schema default, argument omitted.
const DECODE_BYTE_CASES: &[(Option<&str>, &str)] = &[
    (None, "aGVsbG8gd29ybGQ="),
    (Some("base16"), "68656c6c6f20776f726c64"),
    (Some("base32"), "NBSWY3DPEB3W64TMMQ======"),
    (Some("base58"), "StV1DL6CwTryKyV"),
    (Some("base64-nopad"), "aGVsbG8gd29ybGQ"),
    (Some("base64url-nopad"), "aGVsbG8gd29ybGQ"),
    (Some("base32hex"), "D1IMOR3F41RMUSJCCG======"),
    (Some("base36"), "FUVRSIVVNFRBJWAJO"), // case-insensitive input
    (Some("base62"), "AAwf93rvy4aWQVw"),
    (Some("ascii85"), "<~BOu!rD]j7BEbo7~>"),
];

#[tokio::test]
async fn decode_hello_world_across_byte_codecs() {
    let client = connect().await;
    for (format, encoded) in DECODE_BYTE_CASES {
        let args = match format {
            Some(f) => json!({"input": encoded, "format": f}),
            None => json!({"input": encoded}),
        };
        let text = call_text(&client, "decode", args).await;
        assert_eq!(
            text, "hello world",
            "decode with format {format:?} must round-trip to \"hello world\""
        );
    }
    client.cancel().await.ok();
}

// Text codecs (url/html/punycode) decode straight to a string, each with its
// own expected value.
#[tokio::test]
async fn decode_text_codecs() {
    let client = connect().await;
    for (encoded, format, expected) in [
        ("a%20b%26c%3Dd", "url", "a b&c=d"),
        ("x&lt;y&gt;&amp;z", "html", "x<y>&z"),
        ("mnchen-3ya", "punycode", "münchen"),
    ] {
        let args = json!({"input": encoded, "format": format});
        let text = call_text(&client, "decode", args).await;
        assert_eq!(
            text, expected,
            "decode with format {format:?} must yield exactly {expected:?}"
        );
    }
    client.cancel().await.ok();
}

#[tokio::test]
async fn decode_rejects_invalid_base64() {
    let client = connect().await;
    let kind =
        expect_error_kind(&client, "decode", json!({"input": "not-valid-base64!!!"})).await;
    assert_eq!(kind, "std:invalid-args");
    client.cancel().await.ok();
}

/// base64 "//79" decodes to bytes that are not valid UTF-8, so `decode`'s
/// TextOrBytes return value serializes as a CBOR byte string. That is a
/// single structured (object) part, so the MCP bridge projects it into
/// structured_content as {"$bytes": "<base64>"} rather than plain text.
#[tokio::test]
async fn decode_of_non_utf8_bytes_yields_bytes_envelope() {
    let client = connect().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("decode")
                .with_arguments(json!({"input": "//79"}).as_object().unwrap().clone()),
        )
        .await
        .expect("call_tool decode");
    assert_ne!(result.is_error, Some(true), "decode failed: {result:?}");
    assert_eq!(
        result.structured_content.as_ref(),
        Some(&json!({"$bytes": "//79"})),
        "non-UTF-8 output must arrive as the $bytes envelope in structured_content"
    );
    client.cancel().await.ok();
}

/// By contrast, bytes that happen to decode as UTF-8 text serialize as a
/// plain string, not an object — structured_content stays unpopulated.
/// Asserted explicitly so this breaks loudly if that shape ever changes,
/// rather than silently falling back to the weaker text-only assertion.
#[tokio::test]
async fn decode_of_valid_utf8_text_yields_plain_string() {
    let client = connect().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("decode")
                .with_arguments(json!({"input": "aGVsbG8="}).as_object().unwrap().clone()),
        )
        .await
        .expect("call_tool decode");
    assert_ne!(result.is_error, Some(true), "decode failed: {result:?}");
    assert!(
        result.structured_content.is_none(),
        "UTF-8 output must not populate structured_content: {:?}",
        result.structured_content
    );
    assert_eq!(first_text_block(&result).text, "hello");
    client.cancel().await.ok();
}
