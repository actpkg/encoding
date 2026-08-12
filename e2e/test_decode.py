import pytest

# Byte-oriented codecs decoding back to "hello world". Ported from the
# sixteen near-identical blocks in the old decode.hurl; parametrizing is
# where the file actually shrinks relative to it.
BYTE_CASES = [
    (None, "aGVsbG8gd29ybGQ="),  # default (base64)
    ("base16", "68656c6c6f20776f726c64"),
    ("base32", "NBSWY3DPEB3W64TMMQ======"),
    ("base58", "StV1DL6CwTryKyV"),
    ("base64-nopad", "aGVsbG8gd29ybGQ"),
    ("base64url-nopad", "aGVsbG8gd29ybGQ"),
    ("base32hex", "D1IMOR3F41RMUSJCCG======"),
    ("base36", "FUVRSIVVNFRBJWAJO"),  # case-insensitive input
    ("base62", "AAwf93rvy4aWQVw"),
    ("ascii85", "<~BOu!rD]j7BEbo7~>"),
]


@pytest.mark.parametrize("format,encoded", BYTE_CASES)
async def test_decode_hello_world(client, format, encoded):
    args = {"input": encoded}
    if format is not None:
        args["format"] = format
    result = await client.call_tool("decode", args)
    assert result.content[0].text == "hello world"


# Text codecs (url/html/punycode) decode straight to a string, each with its
# own expected value.
TEXT_CASES = [
    ("a%20b%26c%3Dd", "url", "a b&c=d"),
    ("x&lt;y&gt;&amp;z", "html", "x<y>&z"),
    ("mnchen-3ya", "punycode", "münchen"),
]


@pytest.mark.parametrize("encoded,format,expected", TEXT_CASES)
async def test_decode_text_codec(client, encoded, format, expected):
    result = await client.call_tool("decode", {"input": encoded, "format": format})
    assert result.content[0].text == expected


async def test_decode_rejects_invalid_base64(client, expect_error):
    await expect_error(client, "decode", {"input": "not-valid-base64!!!"}, "std:invalid-args")


async def test_decode_of_non_utf8_bytes_yields_bytes_envelope(client):
    # base64 "//79" decodes to bytes that are not valid UTF-8, so `decode`'s
    # TextOrBytes return value serializes as a CBOR byte string. That is a
    # single structured (object) part, so the MCP bridge projects it into
    # structured_content as {"$bytes": "<base64>"} rather than plain text.
    result = await client.call_tool("decode", {"input": "//79"})
    assert result.structured_content == {"$bytes": "//79"}


async def test_decode_of_valid_utf8_text_yields_plain_string(client):
    # By contrast, bytes that happen to decode as UTF-8 text serialize as a
    # plain string, not an object — structured_content stays unpopulated.
    # Asserted explicitly so this breaks loudly if that shape ever changes,
    # rather than silently falling back to the weaker text-only assertion.
    result = await client.call_tool("decode", {"input": "aGVsbG8="})
    assert result.structured_content is None
    assert result.content[0].text == "hello"
