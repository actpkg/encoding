---
name: encoding
description: Encode/decode strings — base64(url), base16, base32(hex), base36/58/62, ascii85, url, quoted-printable, html, punycode
metadata:
  act: {}
---

# Encoding Component

Encode and decode strings. Use for binary data, API tokens, hash outputs, URLs,
HTML, email (MIME), and internationalized domain names.

## Formats

The `format` argument (a fixed set — invalid values are rejected) selects the codec:

| format | notes |
|---|---|
| `base64` | default; standard RFC 4648 alphabet, padded |
| `base64-nopad` | standard alphabet, padding stripped |
| `base64url` | URL-safe alphabet (`-`/`_`), padded |
| `base64url-nopad` | URL-safe alphabet, padding stripped |
| `base16` | hex; lowercase on encode, case-insensitive on decode |
| `base32` | RFC 4648, padded |
| `base32hex` | RFC 4648 extended-hex alphabet, padded |
| `base36` | big-integer, alphabet `0-9a-z`; case-insensitive decode |
| `base58` | Bitcoin alphabet (preserves leading zero bytes) |
| `base62` | big-integer, alphabet `0-9A-Za-z` |
| `ascii85` | Adobe/btoa, wrapped in `<~`…`~>` |
| `url` | percent-encoding |
| `quoted-printable` | MIME (RFC 2045) |
| `html` | HTML entities (`&amp;`, `&lt;`, …) |
| `punycode` | IDNA label encoding |

> Note: `base36` and `base62` are big-integer encodings with no standard
> byte-level leading-zero convention, so leading NUL bytes are not preserved
> on round-trip. Use `base58` if you need leading-zero fidelity.

## Tools

### encode
Encode a string with the given format (default `base64`).

```
encode(input: "hello world")                            → "aGVsbG8gd29ybGQ="
encode(input: "hello world", format: "base64-nopad")    → "aGVsbG8gd29ybGQ"
encode(input: "hello world", format: "base16")          → "68656c6c6f20776f726c64"
encode(input: "hello world", format: "base32")          → "NBSWY3DPEB3W64TMMQ======"
encode(input: "hello world", format: "base32hex")       → "D1IMOR3F41RMUSJCCG======"
encode(input: "hello world", format: "base36")          → "fuvrsivvnfrbjwajo"
encode(input: "hello world", format: "base58")          → "StV1DL6CwTryKyV"
encode(input: "hello world", format: "base62")          → "AAwf93rvy4aWQVw"
encode(input: "hello world", format: "ascii85")         → "<~BOu!rD]j7BEbo7~>"
encode(input: "a b&c=d",     format: "url")             → "a%20b%26c%3Dd"
encode(input: "x<y>&z",      format: "html")            → "x&lt;y&gt;&amp;z"
encode(input: "münchen",     format: "punycode")        → "mnchen-3ya"
```

### decode
Decode a string with the given format (default `base64`).

```
decode(input: "aGVsbG8gd29ybGQ=")                              → "hello world"
decode(input: "D1IMOR3F41RMUSJCCG======", format: "base32hex")→ "hello world"
decode(input: "AAwf93rvy4aWQVw", format: "base62")            → "hello world"
decode(input: "<~BOu!rD]j7BEbo7~>", format: "ascii85")        → "hello world"
decode(input: "a%20b%26c%3Dd", format: "url")                 → "a b&c=d"
decode(input: "mnchen-3ya", format: "punycode")               → "münchen"
```
