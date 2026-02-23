# Advanced Patterns

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
