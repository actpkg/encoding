use act_sdk::prelude::*;
use base64::Engine;

act_sdk::embed_skill!("skill/");

const B32: base32::Alphabet = base32::Alphabet::Rfc4648 { padding: true };
// base-x big-integer alphabets (arbitrary-byte base36/base62).
const B36: &str = "0123456789abcdefghijklmnopqrstuvwxyz";
const B62: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Supported codecs. Serialized as the kebab-case name, so the tool's JSON
/// Schema constrains `format` to exactly these values.
#[derive(Deserialize, JsonSchema, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum Format {
    Base64,
    Base64Nopad,
    Base64url,
    Base64urlNopad,
    Base16,
    Base32,
    Base32hex,
    Base36,
    Base58,
    Base62,
    Ascii85,
    Url,
    QuotedPrintable,
    Html,
    Punycode,
}

/// Input that is either literal UTF-8 text or raw bytes. A plain (CBOR/JSON
/// text) string deserializes to `Text` verbatim; a CBOR byte string (the host's
/// `{"$bytes":"…"}` projection on JSON transports) deserializes to `Bytes`.
enum TextOrBytes {
    Text(String),
    Bytes(Vec<u8>),
}

impl TextOrBytes {
    /// Raw bytes to encode.
    fn as_bytes(&self) -> &[u8] {
        match self {
            TextOrBytes::Text(s) => s.as_bytes(),
            TextOrBytes::Bytes(b) => b,
        }
    }

    /// As a `&str` for text codecs; errors if bytes are not valid UTF-8.
    fn as_str(&self) -> ActResult<&str> {
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

/// Wrap a decoded string as a `text/plain` content-part.
fn text_content(s: String) -> Content {
    Content("text/plain", s.into_bytes())
}

#[act_component]
mod component {
    use super::*;

    /// Encode data.
    #[act_tool(
        description = "Encode a string or binary data. format: base64 (default), base64-nopad, base64url, base64url-nopad, base16, base32, base32hex, base36, base58, base62, ascii85, url, quoted-printable, html, punycode",
        read_only
    )]
    fn encode(
        #[doc = "Data to encode: a text string, or raw bytes as a {\"$bytes\":\"<base64>\"} object"]
        input: TextOrBytes,
        #[doc = "Codec to use (default base64)"] format: Option<Format>,
    ) -> ActResult<String> {
        use base64::engine::general_purpose as b64;
        let bytes = input.as_bytes();
        Ok(match format.unwrap_or(Format::Base64) {
            Format::Base64 => b64::STANDARD.encode(bytes),
            Format::Base64Nopad => b64::STANDARD_NO_PAD.encode(bytes),
            Format::Base64url => b64::URL_SAFE.encode(bytes),
            Format::Base64urlNopad => b64::URL_SAFE_NO_PAD.encode(bytes),
            Format::Base16 => hex::encode(bytes),
            Format::Base32 => base32::encode(B32, bytes),
            Format::Base32hex => data_encoding::BASE32HEX.encode(bytes),
            Format::Base36 => base_x::encode(B36, bytes),
            Format::Base58 => bs58::encode(bytes).into_string(),
            Format::Base62 => base_x::encode(B62, bytes),
            Format::Ascii85 => ascii85::encode(bytes),
            Format::QuotedPrintable => quoted_printable::encode_to_str(bytes),
            // Text codecs operate on the string; require valid UTF-8.
            Format::Url => urlencoding::encode(input.as_str()?).into_owned(),
            Format::Html => html_escape::encode_text(input.as_str()?).into_owned(),
            Format::Punycode => punycode::encode(input.as_str()?)
                .map_err(|_| ActError::invalid_args("Cannot punycode-encode this input"))?,
        })
    }

    /// Decode a string.
    #[act_tool(
        description = "Decode a string. format: base64 (default), base64-nopad, base64url, base64url-nopad, base16, base32, base32hex, base36, base58, base62, ascii85, url, quoted-printable, html, punycode",
        read_only
    )]
    fn decode(
        #[doc = "Encoded string to decode"] input: String,
        #[doc = "Codec the input is in (default base64)"] format: Option<Format>,
    ) -> ActResult<Content> {
        use base64::engine::general_purpose as b64;
        let s = input.trim();
        let bytes = match format.unwrap_or(Format::Base64) {
            Format::Base64 => b64::STANDARD
                .decode(s)
                .map_err(|e| ActError::invalid_args(format!("Invalid base64: {e}")))?,
            Format::Base64Nopad => b64::STANDARD_NO_PAD
                .decode(s)
                .map_err(|e| ActError::invalid_args(format!("Invalid base64-nopad: {e}")))?,
            Format::Base64url => b64::URL_SAFE
                .decode(s)
                .map_err(|e| ActError::invalid_args(format!("Invalid base64url: {e}")))?,
            Format::Base64urlNopad => b64::URL_SAFE_NO_PAD
                .decode(s)
                .map_err(|e| ActError::invalid_args(format!("Invalid base64url-nopad: {e}")))?,
            Format::Base16 => hex::decode(s)
                .map_err(|e| ActError::invalid_args(format!("Invalid base16: {e}")))?,
            Format::Base32 => {
                base32::decode(B32, s).ok_or_else(|| ActError::invalid_args("Invalid base32"))?
            }
            Format::Base32hex => data_encoding::BASE32HEX
                .decode(s.as_bytes())
                .map_err(|e| ActError::invalid_args(format!("Invalid base32hex: {e}")))?,
            Format::Base36 => base_x::decode(B36, &s.to_lowercase())
                .map_err(|e| ActError::invalid_args(format!("Invalid base36: {e}")))?,
            Format::Base58 => bs58::decode(s)
                .into_vec()
                .map_err(|e| ActError::invalid_args(format!("Invalid base58: {e}")))?,
            Format::Base62 => base_x::decode(B62, s)
                .map_err(|e| ActError::invalid_args(format!("Invalid base62: {e}")))?,
            Format::Ascii85 => ascii85::decode(s)
                .map_err(|e| ActError::invalid_args(format!("Invalid ascii85: {e}")))?,
            Format::QuotedPrintable => {
                quoted_printable::decode(s, quoted_printable::ParseMode::Robust)
                    .map_err(|e| ActError::invalid_args(format!("Invalid quoted-printable: {e}")))?
            }
            // Text codecs decode straight to a string → always text/plain.
            Format::Url => {
                return urlencoding::decode(s)
                    .map(|c| text_content(c.into_owned()))
                    .map_err(|e| ActError::invalid_args(format!("Invalid url-encoding: {e}")));
            }
            Format::Html => {
                return Ok(text_content(
                    html_escape::decode_html_entities(&input).into_owned(),
                ));
            }
            Format::Punycode => {
                return punycode::decode(s)
                    .map(text_content)
                    .map_err(|_| ActError::invalid_args("Invalid punycode"));
            }
        };
        // Byte codecs: text/plain if the decoded bytes are valid UTF-8, else octet-stream.
        Ok(match String::from_utf8(bytes) {
            Ok(text) => Content("text/plain", text.into_bytes()),
            Err(e) => Content("application/octet-stream", e.into_bytes()),
        })
    }
}
