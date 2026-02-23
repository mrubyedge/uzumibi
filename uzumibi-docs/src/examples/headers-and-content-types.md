# Headers and Content Types

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
