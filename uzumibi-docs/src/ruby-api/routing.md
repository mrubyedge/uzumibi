# Routing

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
