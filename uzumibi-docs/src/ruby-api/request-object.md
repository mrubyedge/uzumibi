# Request Object

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
