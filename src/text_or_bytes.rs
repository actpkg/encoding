//! `TextOrBytes` — the `encode` tool's input type: either literal text or raw bytes.

use act_sdk::{ActError, ActResult};

/// Input that is either literal UTF-8 text or raw bytes. A plain (CBOR/JSON
/// text) string deserializes to `Text` verbatim; a CBOR byte string (the host's
/// `{"$bytes":"…"}` projection on JSON transports) deserializes to `Bytes`.
pub(crate) enum TextOrBytes {
    Text(String),
    Bytes(Vec<u8>),
}

impl TextOrBytes {
    /// Raw bytes to encode.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        match self {
            TextOrBytes::Text(s) => s.as_bytes(),
            TextOrBytes::Bytes(b) => b,
        }
    }

    /// As a `&str` for text codecs; errors if bytes are not valid UTF-8.
    pub(crate) fn as_str(&self) -> ActResult<&str> {
        match self {
            TextOrBytes::Text(s) => Ok(s),
            TextOrBytes::Bytes(b) => std::str::from_utf8(b)
                .map_err(|e| ActError::invalid_args(format!("input is not valid UTF-8 text: {e}"))),
        }
    }
}

impl<'de> serde::Deserialize<'de> for TextOrBytes {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V;
        impl serde::de::Visitor<'_> for V {
            type Value = TextOrBytes;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a text string or a byte string")
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<TextOrBytes, E> {
                Ok(TextOrBytes::Text(v.to_string()))
            }
            fn visit_string<E: serde::de::Error>(self, v: String) -> Result<TextOrBytes, E> {
                Ok(TextOrBytes::Text(v))
            }
            fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<TextOrBytes, E> {
                Ok(TextOrBytes::Bytes(v.to_vec()))
            }
            fn visit_byte_buf<E: serde::de::Error>(self, v: Vec<u8>) -> Result<TextOrBytes, E> {
                Ok(TextOrBytes::Bytes(v))
            }
        }
        deserializer.deserialize_any(V)
    }
}

impl schemars::JsonSchema for TextOrBytes {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "TextOrBytes".into()
    }
    fn inline_schema() -> bool {
        true
    }
    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        // Same shape as the previous `String` input, so LLM/MCP clients are
        // unaffected; binary is the advanced `{"$bytes":…}` path.
        schemars::json_schema!({ "type": "string" })
    }
}
