# base64 (default)
POST {{baseurl}}/tools/encode
{"arguments": {"input": "hello world"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "aGVsbG8gd29ybGQ="

# base16
POST {{baseurl}}/tools/encode
{"arguments": {"input": "hello world", "format": "base16"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "68656c6c6f20776f726c64"

# base32
POST {{baseurl}}/tools/encode
{"arguments": {"input": "hello world", "format": "base32"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "NBSWY3DPEB3W64TMMQ======"

# base58
POST {{baseurl}}/tools/encode
{"arguments": {"input": "hello world", "format": "base58"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "StV1DL6CwTryKyV"

# url
POST {{baseurl}}/tools/encode
{"arguments": {"input": "a b&c=d", "format": "url"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "a%20b%26c%3Dd"

# html
POST {{baseurl}}/tools/encode
{"arguments": {"input": "x<y>&z", "format": "html"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "x&lt;y&gt;&amp;z"

# punycode
POST {{baseurl}}/tools/encode
{"arguments": {"input": "münchen", "format": "punycode"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "mnchen-3ya"

# base64-nopad
POST {{baseurl}}/tools/encode
{"arguments": {"input": "hello world", "format": "base64-nopad"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "aGVsbG8gd29ybGQ"

# base64url-nopad
POST {{baseurl}}/tools/encode
{"arguments": {"input": "hello world", "format": "base64url-nopad"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "aGVsbG8gd29ybGQ"

# base32hex
POST {{baseurl}}/tools/encode
{"arguments": {"input": "hello world", "format": "base32hex"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "D1IMOR3F41RMUSJCCG======"

# base36
POST {{baseurl}}/tools/encode
{"arguments": {"input": "hello world", "format": "base36"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "fuvrsivvnfrbjwajo"

# base62
POST {{baseurl}}/tools/encode
{"arguments": {"input": "hello world", "format": "base62"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "AAwf93rvy4aWQVw"

# ascii85
POST {{baseurl}}/tools/encode
{"arguments": {"input": "hello world", "format": "ascii85"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "<~BOu!rD]j7BEbo7~>"

# an unknown format is rejected by the schema enum
POST {{baseurl}}/tools/encode
{"arguments": {"input": "hello", "format": "rot13"}}
HTTP 422
[Asserts]
jsonpath "$.error" exists

# binary input via {"$bytes":…} → encode raw bytes (here to base16)
POST {{baseurl}}/tools/encode
{"arguments": {"input": {"$bytes": "//79"}, "format": "base16"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "fffefd"

# plain string input still works (literal text, not base64-decoded)
POST {{baseurl}}/tools/encode
{"arguments": {"input": "hi", "format": "base16"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "6869"
