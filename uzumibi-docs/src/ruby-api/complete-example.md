# Complete Example

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
