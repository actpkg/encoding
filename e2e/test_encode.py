import pytest

# Byte-oriented codecs against one canonical input. Ported from the sixteen
# near-identical blocks in the old encode.hurl; parametrizing is where the
# file actually shrinks relative to it.
BYTE_CASES = [
    (None, "aGVsbG8gd29ybGQ="),  # default (base64)
    ("base16", "68656c6c6f20776f726c64"),
    ("base32", "NBSWY3DPEB3W64TMMQ======"),
    ("base58", "StV1DL6CwTryKyV"),
    ("base64-nopad", "aGVsbG8gd29ybGQ"),
    ("base64url-nopad", "aGVsbG8gd29ybGQ"),
    ("base32hex", "D1IMOR3F41RMUSJCCG======"),
    ("base36", "fuvrsivvnfrbjwajo"),
    ("base62", "AAwf93rvy4aWQVw"),
    ("ascii85", "<~BOu!rD]j7BEbo7~>"),
]


@pytest.mark.parametrize("format,expected", BYTE_CASES)
async def test_encode_hello_world(client, format, expected):
    args = {"input": "hello world"}
    if format is not None:
        args["format"] = format
    result = await client.call_tool("encode", args)
    assert result.content[0].text == expected


# Text codecs (url/html/punycode) operate on the string itself, so they each
# need their own input rather than sharing "hello world".
TEXT_CASES = [
    ("a b&c=d", "url", "a%20b%26c%3Dd"),
    ("x<y>&z", "html", "x&lt;y&gt;&amp;z"),
    ("münchen", "punycode", "mnchen-3ya"),
]


@pytest.mark.parametrize("text,format,expected", TEXT_CASES)
async def test_encode_text_codec(client, text, format, expected):
    result = await client.call_tool("encode", {"input": text, "format": format})
    assert result.content[0].text == expected


async def test_encode_rejects_unknown_format(client, expect_error):
    # `format` is a schema enum; an unrecognized value fails to deserialize
    # before `encode`'s body ever runs, which surfaces as std:invalid-args —
    # same kind as an explicit ActError::invalid_args call (measured via
    # act-sdk-macros' generated argument-deserialization arm).
    await expect_error(client, "encode", {"input": "hello", "format": "rot13"}, "std:invalid-args")


async def test_encode_binary_input_via_bytes_envelope(client):
    # A {"$bytes": "<base64>"} object is the transport's byte-string
    # projection: encode should treat it as raw bytes, not literal text.
    result = await client.call_tool("encode", {"input": {"$bytes": "//79"}, "format": "base16"})
    assert result.content[0].text == "fffefd"


async def test_encode_plain_string_input_is_literal_text(client):
    # A bare string is literal text to encode, not base64-decoded first.
    result = await client.call_tool("encode", {"input": "hi", "format": "base16"})
    assert result.content[0].text == "6869"
