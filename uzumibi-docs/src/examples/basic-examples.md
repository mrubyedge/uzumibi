# Basic Examples

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
