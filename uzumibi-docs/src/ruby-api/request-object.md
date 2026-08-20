# Request Object

Route handlers receive an `Uzumibi::Request` as `req`.

| Property | Value |
| --- | --- |
| `req.method` | HTTP method String |
| `req.path` | Request pathname |
| `req.headers` | Header Hash with String keys and values |
| `req.params` | Path, query, and parsed body parameters with Symbol keys |
| `req.body` | Parsed JSON value when supported, otherwise the raw body String |
| `req.raw_body` | Raw request body as a Ruby String |
| `req.cookie` | Parsed Cookie header as a Hash with String keys |

## Parameters

~~~ruby
get "/users/:id" do |req, res|
  id = req.params[:id]
  verbose = req.params[:verbose]
  res.return(200, {}, "#{id}: #{verbose}")
end
~~~

Path parameters are merged first, followed by query parameters and then supported body parameters. A later source replaces an earlier value with the same key.

## JSON bodies

When the content type is exactly `application/json` and JSON support is enabled by the template, valid JSON is assigned to `req.body`. Top-level object fields are also merged into `req.params`.

~~~ruby
post "/users" do |req, res|
  data = req.body
  name = data["name"]
  res.return(
    201,
    { "content-type" => "application/json" },
    JSON.generate({ "created" => name })
  )
end
~~~

If parsing fails, `req.body` remains the raw String. Use `req.raw_body` when the original payload is required regardless of content type.

## Form bodies

For an exact `application/x-www-form-urlencoded` content type, decoded form fields are merged into `req.params`.

## Headers

Header casing and filtering depend on the platform adapter. The Cloudflare adapter currently passes lowercase Workers header names but omits `cf-connecting-ip`, `cf-ray`, and names beginning with `x-`.
