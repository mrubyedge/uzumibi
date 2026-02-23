# Examples

This section provides practical examples of Uzumibi applications for common use cases.

## Basic Examples

### Hello World

The simplest Uzumibi application:

```ruby
class App < Uzumibi::Router
  get "/" do |req, res|
    res.status_code = 200
    res.headers = { "Content-Type" => "text/plain" }
    res.body = "Hello, World!"
    res
  end
end

$APP = App.new
```

### Path Parameters

Extract parameters from the URL path:

```ruby
class App < Uzumibi::Router
  get "/greet/:name" do |req, res|
    name = req.params[:name]
    
    res.status_code = 200
    res.headers = { "Content-Type" => "text/plain" }
    res.body = "Hello, #{name}!"
    res
  end
  
  get "/users/:user_id/posts/:post_id" do |req, res|
    user_id = req.params[:user_id]
    post_id = req.params[:post_id]
    
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      user_id: user_id,
      post_id: post_id
    })
    res
  end
end

$APP = App.new
```

### Query Parameters

Access URL query parameters:

```ruby
class App < Uzumibi::Router
  get "/search" do |req, res|
    query = req.params[:q] || ""
    page = (req.params[:page] || "1").to_i
    limit = (req.params[:limit] || "10").to_i
    
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      query: query,
      page: page,
      limit: limit,
      results: []  # Add your search logic here
    })
    res
  end
end

$APP = App.new
```

## HTTP Methods

### GET, POST, PUT, DELETE

Handle different HTTP methods:

```ruby
class App < Uzumibi::Router
  # GET - Retrieve resource
  get "/users/:id" do |req, res|
    user_id = req.params[:id]
    
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      id: user_id,
      name: "User #{user_id}",
      email: "user#{user_id}@example.com"
    })
    res
  end
  
  # POST - Create resource
  post "/users" do |req, res|
    data = JSON.parse(req.body)
    
    res.status_code = 201
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      id: rand(1000),
      name: data["name"],
      email: data["email"],
      created: true
    })
    res
  end
  
  # PUT - Update resource
  put "/users/:id" do |req, res|
    user_id = req.params[:id]
    data = JSON.parse(req.body)
    
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      id: user_id,
      name: data["name"],
      updated: true
    })
    res
  end
  
  # DELETE - Delete resource
  delete "/users/:id" do |req, res|
    user_id = req.params[:id]
    
    res.status_code = 204
    res.body = ""
    res
  end
end

$APP = App.new
```

## JSON API

### RESTful API Example

A complete RESTful API example:

```ruby
class App < Uzumibi::Router
  # List all items
  get "/api/items" do |req, res|
    page = (req.params[:page] || "1").to_i
    
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      page: page,
      items: [
        { id: 1, name: "Item 1" },
        { id: 2, name: "Item 2" }
      ]
    })
    res
  end
  
  # Get single item
  get "/api/items/:id" do |req, res|
    item_id = req.params[:id].to_i
    
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      id: item_id,
      name: "Item #{item_id}",
      description: "Description for item #{item_id}"
    })
    res
  end
  
  # Create item
  post "/api/items" do |req, res|
    begin
      data = JSON.parse(req.body)
      
      # Validate
      if !data["name"] || data["name"].empty?
        res.status_code = 400
        res.body = JSON.generate({ error: "Name is required" })
      else
        res.status_code = 201
        res.headers = {
          "Content-Type" => "application/json",
          "Location" => "/api/items/#{rand(1000)}"
        }
        res.body = JSON.generate({
          id: rand(1000),
          name: data["name"],
          description: data["description"]
        })
      end
    rescue JSON::ParserError
      res.status_code = 400
      res.body = JSON.generate({ error: "Invalid JSON" })
    end
    res
  end
  
  # Update item
  put "/api/items/:id" do |req, res|
    item_id = req.params[:id].to_i
    
    begin
      data = JSON.parse(req.body)
      
      res.status_code = 200
      res.headers = { "Content-Type" => "application/json" }
      res.body = JSON.generate({
        id: item_id,
        name: data["name"],
        description: data["description"],
        updated: true
      })
    rescue JSON::ParserError
      res.status_code = 400
      res.body = JSON.generate({ error: "Invalid JSON" })
    end
    res
  end
  
  # Delete item
  delete "/api/items/:id" do |req, res|
    res.status_code = 204
    res.body = ""
    res
  end
end

$APP = App.new
```

## Form Handling

### Processing Form Data

Handle form submissions:

```ruby
class App < Uzumibi::Router
  # Show form (HTML)
  get "/form" do |req, res|
    res.status_code = 200
    res.headers = { "Content-Type" => "text/html" }
    res.body = <<~HTML
      <!DOCTYPE html>
      <html>
      <head><title>Form Example</title></head>
      <body>
        <h1>Submit Form</h1>
        <form method="POST" action="/form">
          <label>Name: <input type="text" name="name"></label><br>
          <label>Email: <input type="email" name="email"></label><br>
          <button type="submit">Submit</button>
        </form>
      </body>
      </html>
    HTML
    res
  end
  
  # Process form submission
  post "/form" do |req, res|
    # Form data is automatically parsed into req.params
    # when Content-Type is application/x-www-form-urlencoded
    name = req.params[:name]
    email = req.params[:email]
    
    res.status_code = 200
    res.headers = { "Content-Type" => "text/html" }
    res.body = <<~HTML
      <!DOCTYPE html>
      <html>
      <head><title>Form Submitted</title></head>
      <body>
        <h1>Thank you!</h1>
        <p>Name: #{name}</p>
        <p>Email: #{email}</p>
      </body>
      </html>
    HTML
    res
  end
end

$APP = App.new
```

## Redirects

### Redirect Examples

```ruby
class App < Uzumibi::Router
  # Temporary redirect (302)
  get "/old-path" do |req, res|
    res.status_code = 302
    res.headers = {
      "Location" => "/new-path",
      "Content-Type" => "text/plain"
    }
    res.body = "Redirecting..."
    res
  end
  
  # Permanent redirect (301)
  get "/moved" do |req, res|
    res.status_code = 301
    res.headers = {
      "Location" => "/permanently-moved",
      "Content-Type" => "text/plain"
    }
    res.body = "Moved Permanently"
    res
  end
  
  # Redirect with parameters
  get "/user/:id" do |req, res|
    user_id = req.params[:id]
    res.status_code = 302
    res.headers = { "Location" => "/users/#{user_id}/profile" }
    res.body = ""
    res
  end
end

$APP = App.new
```

## Error Handling

### Custom Error Responses

```ruby
class App < Uzumibi::Router
  get "/error-demo" do |req, res|
    error_type = req.params[:type]
    
    case error_type
    when "404"
      res.status_code = 404
      res.headers = { "Content-Type" => "application/json" }
      res.body = JSON.generate({
        error: "Not Found",
        message: "The requested resource was not found"
      })
    when "500"
      res.status_code = 500
      res.headers = { "Content-Type" => "application/json" }
      res.body = JSON.generate({
        error: "Internal Server Error",
        message: "Something went wrong"
      })
    when "401"
      res.status_code = 401
      res.headers = {
        "Content-Type" => "application/json",
        "WWW-Authenticate" => "Bearer"
      }
      res.body = JSON.generate({
        error: "Unauthorized",
        message: "Authentication required"
      })
    else
      res.status_code = 200
      res.body = "Specify ?type=404|500|401"
    end
    res
  end
  
  # Handle errors in route
  post "/api/data" do |req, res|
    begin
      data = JSON.parse(req.body)
      
      # Process data...
      
      res.status_code = 200
      res.headers = { "Content-Type" => "application/json" }
      res.body = JSON.generate({ success: true })
    rescue JSON::ParserError => e
      res.status_code = 400
      res.headers = { "Content-Type" => "application/json" }
      res.body = JSON.generate({
        error: "Bad Request",
        message: "Invalid JSON: #{e.message}"
      })
    rescue => e
      debug_console("[ERROR] #{e.message}")
      res.status_code = 500
      res.headers = { "Content-Type" => "application/json" }
      res.body = JSON.generate({
        error: "Internal Server Error",
        message: "An unexpected error occurred"
      })
    end
    res
  end
end

$APP = App.new
```

## Headers and Content Types

### Working with Headers

```ruby
class App < Uzumibi::Router
  # Return JSON
  get "/json" do |req, res|
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({ message: "Hello JSON" })
    res
  end
  
  # Return HTML
  get "/html" do |req, res|
    res.status_code = 200
    res.headers = { "Content-Type" => "text/html; charset=utf-8" }
    res.body = "<html><body><h1>Hello HTML</h1></body></html>"
    res
  end
  
  # Return plain text
  get "/text" do |req, res|
    res.status_code = 200
    res.headers = { "Content-Type" => "text/plain; charset=utf-8" }
    res.body = "Hello Plain Text"
    res
  end
  
  # Custom headers
  get "/custom-headers" do |req, res|
    res.status_code = 200
    res.headers = {
      "Content-Type" => "text/plain",
      "X-Custom-Header" => "CustomValue",
      "X-Request-ID" => "#{Time.now.to_i}",
      "Cache-Control" => "public, max-age=3600",
      "X-Powered-By" => "Uzumibi/#{RUBY_VERSION}"
    }
    res.body = "Check the response headers!"
    res
  end
  
  # Read request headers
  get "/echo-headers" do |req, res|
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      user_agent: req.headers["user-agent"],
      accept: req.headers["accept"],
      host: req.headers["host"]
    })
    res
  end
end

$APP = App.new
```

## Advanced Patterns

### API Versioning

```ruby
class App < Uzumibi::Router
  # Version 1 API
  get "/api/v1/users" do |req, res|
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      version: "1.0",
      users: [{ id: 1, name: "User 1" }]
    })
    res
  end
  
  # Version 2 API (with additional fields)
  get "/api/v2/users" do |req, res|
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      version: "2.0",
      users: [{
        id: 1,
        name: "User 1",
        email: "user1@example.com",
        created_at: Time.now.to_i
      }]
    })
    res
  end
end

$APP = App.new
```

### Content Negotiation

```ruby
class App < Uzumibi::Router
  get "/data" do |req, res|
    accept = req.headers["accept"] || "application/json"
    
    data = { message: "Hello", timestamp: Time.now.to_i }
    
    if accept.include?("application/json")
      res.status_code = 200
      res.headers = { "Content-Type" => "application/json" }
      res.body = JSON.generate(data)
    elsif accept.include?("text/html")
      res.status_code = 200
      res.headers = { "Content-Type" => "text/html" }
      res.body = "<html><body><h1>#{data[:message]}</h1><p>Time: #{data[:timestamp]}</p></body></html>"
    else
      res.status_code = 200
      res.headers = { "Content-Type" => "text/plain" }
      res.body = "Message: #{data[:message]}\nTime: #{data[:timestamp]}"
    end
    res
  end
end

$APP = App.new
```

## Real-World Example

### Complete Blog API

A more complete example showing a blog API:

```ruby
class App < Uzumibi::Router
  # Root endpoint
  get "/" do |req, res|
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      name: "Blog API",
      version: "1.0",
      endpoints: {
        posts: "/api/posts",
        authors: "/api/authors"
      }
    })
    res
  end
  
  # List posts
  get "/api/posts" do |req, res|
    page = (req.params[:page] || "1").to_i
    tag = req.params[:tag]
    
    posts = [
      { id: 1, title: "First Post", author_id: 1, tags: ["ruby", "web"] },
      { id: 2, title: "Second Post", author_id: 2, tags: ["edge", "wasm"] }
    ]
    
    # Filter by tag if provided
    posts = posts.select { |p| p[:tags].include?(tag) } if tag
    
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      page: page,
      posts: posts
    })
    res
  end
  
  # Get single post
  get "/api/posts/:id" do |req, res|
    post_id = req.params[:id].to_i
    
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      id: post_id,
      title: "Post #{post_id}",
      content: "Content for post #{post_id}",
      author_id: 1,
      created_at: Time.now.to_i
    })
    res
  end
  
  # Create post
  post "/api/posts" do |req, res|
    begin
      data = JSON.parse(req.body)
      
      if !data["title"] || data["title"].empty?
        res.status_code = 400
        res.body = JSON.generate({ error: "Title is required" })
      else
        new_id = rand(1000)
        res.status_code = 201
        res.headers = {
          "Content-Type" => "application/json",
          "Location" => "/api/posts/#{new_id}"
        }
        res.body = JSON.generate({
          id: new_id,
          title: data["title"],
          content: data["content"],
          author_id: data["author_id"],
          created_at: Time.now.to_i
        })
      end
    rescue JSON::ParserError
      res.status_code = 400
      res.body = JSON.generate({ error: "Invalid JSON" })
    end
    res
  end
  
  # List authors
  get "/api/authors" do |req, res|
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      authors: [
        { id: 1, name: "Alice", email: "alice@example.com" },
        { id: 2, name: "Bob", email: "bob@example.com" }
      ]
    })
    res
  end
  
  # Get single author
  get "/api/authors/:id" do |req, res|
    author_id = req.params[:id].to_i
    
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      id: author_id,
      name: "Author #{author_id}",
      email: "author#{author_id}@example.com",
      bio: "Biography for author #{author_id}"
    })
    res
  end
end

$APP = App.new
```

## Next Steps

- Explore the [Ruby API Reference](./ruby-api.md) for detailed API documentation
- Check [Supported Platforms](./platforms.md) for platform-specific examples
- Review [External Service Abstractions](./external-services.md) for using KV stores, caches, etc.
