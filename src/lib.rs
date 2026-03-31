use act_sdk::prelude::*;
use base64::Engine;

act_sdk::embed_skill!("skill/");

#[act_component]
mod component {
    use super::*;

    /// Encode data.
    #[act_tool(description = "Encode a string to base64 or hex", read_only)]
    fn encode(
        #[doc = "String to encode"] input: String,
        #[doc = "Format: 'base64' (default) or 'hex'"] format: Option<String>,
    ) -> ActResult<String> {
        match format.as_deref().unwrap_or("base64") {
            "base64" => Ok(base64::engine::general_purpose::STANDARD.encode(input.as_bytes())),
            "hex" => Ok(hex::encode(input.as_bytes())),
            other => Err(ActError::invalid_args(format!(
                "Unknown format: {other}. Use: base64, hex"
            ))),
        }
    }

    /// Decode data.
    #[act_tool(description = "Decode a base64 or hex string", read_only)]
    fn decode(
        #[doc = "Encoded string to decode"] input: String,
        #[doc = "Format: 'base64' (default) or 'hex'"] format: Option<String>,
    ) -> ActResult<String> {
        let bytes = match format.as_deref().unwrap_or("base64") {
            "base64" => base64::engine::general_purpose::STANDARD
                .decode(input.trim())
                .map_err(|e| ActError::invalid_args(format!("Invalid base64: {e}")))?,
            "hex" => hex::decode(input.trim())
                .map_err(|e| ActError::invalid_args(format!("Invalid hex: {e}")))?,
            other => {
                return Err(ActError::invalid_args(format!(
                    "Unknown format: {other}. Use: base64, hex"
                )));
            }
        };
        String::from_utf8(bytes)
            .map_err(|e| ActError::invalid_args(format!("Decoded bytes are not valid UTF-8: {e}")))
    }
}
