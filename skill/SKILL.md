---
name: encoding
description: Base64 and hex encoding/decoding
metadata:
  act: {}
---

# Encoding Component

Encode and decode strings. Use for binary data, API tokens, hash outputs.

## Tools

### encode
Encode a string to base64 or hex.

```
encode(input: "hello world")                 → "aGVsbG8gd29ybGQ="
encode(input: "hello world", format: "hex")  → "68656c6c6f20776f726c64"
```

### decode
Decode a base64 or hex string back to text.

```
decode(input: "aGVsbG8gd29ybGQ=")                → "hello world"
decode(input: "68656c6c6f20776f726c64", format: "hex") → "hello world"
```
