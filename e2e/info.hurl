GET {{baseurl}}/info
HTTP 200
[Asserts]
jsonpath "$['std:name']" == "encoding"
jsonpath "$['std:version']" isString
