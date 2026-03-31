# Base64 decode (default)
POST {{baseurl}}/tools/decode
{"arguments": {"input": "aGVsbG8gd29ybGQ="}}
HTTP 200
[Asserts]
jsonpath "$.content[0].text" == "hello world"

# Hex decode
POST {{baseurl}}/tools/decode
{"arguments": {"input": "68656c6c6f20776f726c64", "format": "hex"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].text" == "hello world"

# Invalid base64
POST {{baseurl}}/tools/decode
{"arguments": {"input": "not-valid-base64!!!"}}
HTTP 200
[Asserts]
jsonpath "$.error" exists
