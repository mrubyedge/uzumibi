# Ruby API Reference

This section describes the Ruby API provided by Uzumibi for building edge applications.

## Routing

Uzumibi provides a Sinatra-inspired routing DSL. Your application class should inherit from `Uzumibi::Router`.

### Defining Routes

Routes are defined using HTTP method names as class methods:

```ruby
class App < Uzumibi::Router
  get "/path" do |req, res|
    # Handle GET request
  end

  post "/path" do |req, res|
    # Handle POST request
  end

  put "/path" do |req, res|
    # Handle PUT request
  end

  delete "/path" do |req, res|
    # Handle DELETE request
  end

  head "/path" do |req, res|
    # Handle HEAD request (body will be automatically cleared)
  end
end
```

### Path Parameters

You can define dynamic path segments using the `:param` syntax:

```ruby
class App < Uzumibi::Router
  get "/users/:id" do |req, res|
    user_id = req.params[:id]
    res.status_code = 200
    res.headers = { "Content-Type" => "text/plain" }
    res.body = "User ID: #{user_id}"
    res
  end

  get "/posts/:post_id/comments/:comment_id" do |req, res|
    post_id = req.params[:post_id]
    comment_id = req.params[:comment_id]
    res.status_code = 200
    res.body = "Post: #{post_id}, Comment: #{comment_id}"
    res
  end
end
```

### Query Parameters

Query parameters are automatically parsed and available in `req.params`:

```ruby
get "/search" do |req, res|
  query = req.params[:q]
  page = req.params[:page] || "1"
  
  res.status_code = 200
  res.body = "Search: #{query}, Page: #{page}"
  res
end
```

For a request to `/search?q=ruby&page=2`, `req.params` will contain both `:q` and `:page`.

## Request Object

The request object (`req`) provides access to all request data.

### Properties

#### `req.method`

The HTTP method as a string:

```ruby
get "/debug" do |req, res|
  res.body = "Method: #{req.method}"  # => "GET"
  res
end
```

#### `req.path`

The request path:

```ruby
get "/users/:id" do |req, res|
  res.body = "Path: #{req.path}"  # => "/users/123"
  res
end
```

#### `req.headers`

A hash of request headers (keys are lowercase):

```ruby
get "/" do |req, res|
  user_agent = req.headers["user-agent"]
  content_type = req.headers["content-type"]
  
  res.body = "UA: #{user_agent}"
  res
end
```

#### `req.params`

A hash containing both path parameters and query parameters:

```ruby
get "/greet/:name" do |req, res|
  # For request: /greet/alice?title=Dr
  name = req.params[:name]    # => "alice"
  title = req.params[:title]  # => "Dr"
  
  res.body = "Hello, #{title} #{name}!"
  res
end
```

#### `req.body`

The request body as a string:

```ruby
post "/data" do |req, res|
  data = req.body
  res.status_code = 200
  res.body = "Received: #{data}"
  res
end
```

For `application/json` content type, you can parse it:

```ruby
post "/api/users" do |req, res|
  begin
    data = JSON.parse(req.body)
    name = data["name"]
    
    res.status_code = 201
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({ created: name })
  rescue => e
    res.status_code = 400
    res.body = "Invalid JSON"
  end
  res
end
```

For `application/x-www-form-urlencoded` content type, form data is automatically parsed into `req.params`.

## Response Object

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

## Complete Example

Here's a complete example demonstrating the API:

```ruby
class App < Uzumibi::Router
  # Simple GET route
  get "/" do |req, res|
    res.status_code = 200
    res.headers = {
      "Content-Type" => "text/plain",
      "X-Powered-By" => "Uzumibi"
    }
    res.body = "Welcome to Uzumibi!"
    res
  end

  # Path parameters
  get "/users/:id" do |req, res|
    user_id = req.params[:id]
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({ id: user_id, name: "User #{user_id}" })
    res
  end

  # Query parameters
  get "/search" do |req, res|
    query = req.params[:q] || ""
    res.status_code = 200
    res.body = "Searching for: #{query}"
    res
  end

  # POST with body
  post "/api/data" do |req, res|
    debug_console("[Uzumibi] Received: #{req.body}")
    
    res.status_code = 200
    res.headers = { "Content-Type" => "text/plain" }
    res.body = "Received #{req.body.size} bytes"
    res
  end

  # Redirect
  get "/old-path" do |req, res|
    res.status_code = 301
    res.headers = { "Location" => "/new-path" }
    res.body = "Moved"
    res
  end

  # Error response
  get "/error" do |req, res|
    res.status_code = 500
    res.headers = { "Content-Type" => "text/plain" }
    res.body = "Internal Server Error"
    res
  end
end

$APP = App.new
```

## Helper Functions

### `debug_console(message)`

Output debug messages to the console:

```ruby
get "/debug" do |req, res|
  debug_console("[Debug] Request received: #{req.path}")
  debug_console("[Debug] Headers: #{req.headers.inspect}")
  
  res.status_code = 200
  res.body = "Check console for debug output"
  res
end
```

Note: The exact behavior of `debug_console` depends on the platform:
- Cloudflare Workers: Outputs to Workers console
- Fastly Compute: Outputs to Fastly logs
- Local development: Outputs to stdout

## Error Handling

If a route is not found, Uzumibi automatically returns a 404 response:

```
Status: 404 Not Found
Body: "Not Found"
```

To handle errors in your route:

```ruby
post "/api/data" do |req, res|
  begin
    data = JSON.parse(req.body)
    # Process data...
    
    res.status_code = 200
    res.body = "Success"
  rescue JSON::ParserError => e
    res.status_code = 400
    res.body = "Invalid JSON: #{e.message}"
  rescue => e
    res.status_code = 500
    res.body = "Error: #{e.message}"
  end
  res
end
```

## Best Practices

1. **Always return `res`**: Make sure to return the response object from every route handler
2. **Set Content-Type**: Always set appropriate `Content-Type` header
3. **Use appropriate status codes**: Return correct HTTP status codes for different scenarios
4. **Validate input**: Check and validate request parameters and body
5. **Keep routes simple**: Complex logic should be extracted into helper methods or classes
6. **Use debug_console carefully**: Excessive logging can impact performance

## Limitations

Due to the constrained edge environment and mruby limitations:

- No file system access
- Limited Ruby standard library
- No native C extensions
- Memory constraints vary by platform
- No background jobs or async processing within a request

See [Supported Platforms](./platforms.md) for platform-specific limitations and features.
