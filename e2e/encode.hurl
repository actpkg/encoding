# Base64 encode (default)
POST {{baseurl}}/tools/encode
{"arguments": {"input": "hello world"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "aGVsbG8gd29ybGQ="

# Hex encode
POST {{baseurl}}/tools/encode
{"arguments": {"input": "hello world", "format": "hex"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" == "68656c6c6f20776f726c64"
