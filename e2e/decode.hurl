# base64 (default)
POST {{baseurl}}/tools/decode
{"arguments": {"input": "aGVsbG8gd29ybGQ="}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "hello world"

# base16
POST {{baseurl}}/tools/decode
{"arguments": {"input": "68656c6c6f20776f726c64", "format": "base16"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "hello world"

# base32
POST {{baseurl}}/tools/decode
{"arguments": {"input": "NBSWY3DPEB3W64TMMQ======", "format": "base32"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "hello world"

# base58
POST {{baseurl}}/tools/decode
{"arguments": {"input": "StV1DL6CwTryKyV", "format": "base58"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "hello world"

# url
POST {{baseurl}}/tools/decode
{"arguments": {"input": "a%20b%26c%3Dd", "format": "url"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "a b&c=d"

# html
POST {{baseurl}}/tools/decode
{"arguments": {"input": "x&lt;y&gt;&amp;z", "format": "html"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "x<y>&z"

# punycode
POST {{baseurl}}/tools/decode
{"arguments": {"input": "mnchen-3ya", "format": "punycode"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "münchen"

# base64-nopad
POST {{baseurl}}/tools/decode
{"arguments": {"input": "aGVsbG8gd29ybGQ", "format": "base64-nopad"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "hello world"

# base64url-nopad
POST {{baseurl}}/tools/decode
{"arguments": {"input": "aGVsbG8gd29ybGQ", "format": "base64url-nopad"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "hello world"

# base32hex
POST {{baseurl}}/tools/decode
{"arguments": {"input": "D1IMOR3F41RMUSJCCG======", "format": "base32hex"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "hello world"

# base36 (case-insensitive)
POST {{baseurl}}/tools/decode
{"arguments": {"input": "FUVRSIVVNFRBJWAJO", "format": "base36"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "hello world"

# base62
POST {{baseurl}}/tools/decode
{"arguments": {"input": "AAwf93rvy4aWQVw", "format": "base62"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "hello world"

# ascii85
POST {{baseurl}}/tools/decode
{"arguments": {"input": "<~BOu!rD]j7BEbo7~>", "format": "ascii85"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "hello world"

# invalid base64
POST {{baseurl}}/tools/decode
{"arguments": {"input": "not-valid-base64!!!"}}
HTTP 422
[Asserts]
jsonpath "$.error" exists

# base64 of non-UTF-8 bytes → a {"$bytes"} envelope (was a 422 before)
POST {{baseurl}}/tools/decode
{"arguments": {"input": "//79"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data['$bytes']" == "//79"

# base64 of valid UTF-8 text → a plain string
POST {{baseurl}}/tools/decode
{"arguments": {"input": "aGVsbG8="}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "hello"
