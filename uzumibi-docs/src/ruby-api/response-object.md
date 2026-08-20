# Response Object

Route handlers receive an `Uzumibi::Response` as `res`.

Set all three response properties:

~~~ruby
get "/" do |req, res|
  res.status_code = 200
  res.headers = { "content-type" => "text/plain" }
  res.body = "Hello\n"
  res
end
~~~

| Property | Required type |
| --- | --- |
| `res.status_code` | Integer representable as an HTTP status |
| `res.headers` | Hash of String-compatible keys and values |
| `res.body` | Ruby String |

## `res.return`

`res.return(status_code, headers, body)` assigns all fields and returns the response object:

~~~ruby
get "/health" do |req, res|
  res.return(200, { "content-type" => "text/plain" }, "ok\n")
end
~~~

The router uses the response object passed to the handler. Ending a handler with `res` is the conventional style, while `res.return` is useful for concise handlers.

## Encoding

The core response transport serializes the bytes of the Ruby String. Individual platform hosts decide how those bytes become a platform response. The current Cloudflare JavaScript adapter decodes the body as text, so it is not yet a transparent arbitrary-binary response path.
