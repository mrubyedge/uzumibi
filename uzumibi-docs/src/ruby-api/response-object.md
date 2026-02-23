# Response Object

The response object (`res`) is used to build the HTTP response.

### Properties

#### `res.status_code`

Set the HTTP status code:

```ruby
get "/ok" do |req, res|
  res.status_code = 200
  res
end

get "/not-found" do |req, res|
  res.status_code = 404
  res.body = "Not Found"
  res
end

post "/created" do |req, res|
  res.status_code = 201
  res.body = "Created"
  res
end
```

Common status codes:
- `200` - OK
- `201` - Created
- `204` - No Content
- `301` - Moved Permanently
- `302` - Found (Redirect)
- `400` - Bad Request
- `401` - Unauthorized
- `403` - Forbidden
- `404` - Not Found
- `500` - Internal Server Error

#### `res.headers`

Set response headers as a hash:

```ruby
get "/" do |req, res|
  res.status_code = 200
  res.headers = {
    "Content-Type" => "text/html",
    "X-Custom-Header" => "value",
    "Cache-Control" => "public, max-age=3600"
  }
  res.body = "<h1>Hello</h1>"
  res
end
```

#### `res.body`

Set the response body as a string:

```ruby
get "/json" do |req, res|
  res.status_code = 200
  res.headers = { "Content-Type" => "application/json" }
  res.body = JSON.generate({
    message: "Hello",
    timestamp: Time.now.to_i
  })
  res
end
```

### Returning the Response

Always return the `res` object from the route handler:

```ruby
get "/example" do |req, res|
  res.status_code = 200
  res.body = "Example"
  res  # Important: return the response object
end
```
