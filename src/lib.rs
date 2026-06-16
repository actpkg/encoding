use act_sdk::prelude::*;
use base64::Engine;

mod text_or_bytes;
use text_or_bytes::TextOrBytes;

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
    ) -> ActResult<TextOrBytes> {
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
            // Text codecs decode straight to a string.
            Format::Url => {
                return urlencoding::decode(s)
                    .map(|c| TextOrBytes::Text(c.into_owned()))
                    .map_err(|e| ActError::invalid_args(format!("Invalid url-encoding: {e}")));
            }
            Format::Html => {
                return Ok(TextOrBytes::Text(
                    html_escape::decode_html_entities(&input).into_owned(),
                ));
            }
            Format::Punycode => {
                return punycode::decode(s)
                    .map(TextOrBytes::Text)
                    .map_err(|_| ActError::invalid_args("Invalid punycode"));
            }
        };
        // Byte codecs: return the decoded bytes; TextOrBytes serializes valid
        // UTF-8 as a text string and anything else as a {"$bytes"} envelope.
        Ok(TextOrBytes::Bytes(bytes))
    }
}
