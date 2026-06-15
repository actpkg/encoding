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

impl serde::Serialize for TextOrBytes {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Content that is valid UTF-8 serializes as a text string; otherwise as a
        // byte string (the `{"$bytes":…}` envelope on JSON transports).
        match self {
            TextOrBytes::Text(s) => serializer.serialize_str(s),
            TextOrBytes::Bytes(b) => match std::str::from_utf8(b) {
                Ok(s) => serializer.serialize_str(s),
                Err(_) => serializer.serialize_bytes(b),
            },
        }
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
        // Either literal UTF-8 text (a plain string) or raw bytes. Binary reaches
        // this type only as the canonical `{"$bytes":"<base64>"}` object (a bare
        // string is always text), so that is the second `oneOf` branch.
        schemars::json_schema!({
            "oneOf": [
                { "type": "string", "description": "Literal UTF-8 text to encode." },
                {
                    "type": "object",
                    "description": "Raw bytes to encode, as a base64 byte-string wrapper.",
                    "properties": {
                        "$bytes": { "type": "string", "contentEncoding": "base64" }
                    },
                    "required": ["$bytes"],
                    "additionalProperties": false
                }
            ]
        })
    }
}
