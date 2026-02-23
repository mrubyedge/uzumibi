# Usage Examples

### Example 1: KV-Backed API

```ruby
class App < Uzumibi::Router
  get "/counter" do |req, res|
    # Get current count
    count = Uzumibi::KV.get("counter")
    count = count ? count.to_i : 0
    
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({ count: count })
    res
  end
  
  post "/counter/increment" do |req, res|
    # Increment counter
    count = Uzumibi::KV.get("counter")
    count = count ? count.to_i + 1 : 1
    Uzumibi::KV.put("counter", count.to_s)
    
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({ count: count })
    res
  end
end
```

### Example 2: Cached API Response

```ruby
class App < Uzumibi::Router
  get "/weather/:city" do |req, res|
    city = req.params[:city]
    cache_key = "weather:#{city}"
    
    # Try to get from cache
    cached = Uzumibi::Cache.get(cache_key)
    
    if cached
      res.status_code = 200
      res.headers = { 
        "Content-Type" => "application/json",
        "X-Cache" => "HIT"
      }
      res.body = cached
    else
      # Fetch from external API
      weather_res = Uzumibi::Fetch.get(
        "https://api.weather.com/#{city}"
      )
      
      # Cache for 1 hour
      Uzumibi::Cache.put(cache_key, weather_res.body, ttl: 3600)
      
      res.status_code = 200
      res.headers = { 
        "Content-Type" => "application/json",
        "X-Cache" => "MISS"
      }
      res.body = weather_res.body
    end
    
    res
  end
end
```

### Example 3: Database-Backed Application

```ruby
class App < Uzumibi::Router
  get "/users/:id" do |req, res|
    user_id = req.params[:id].to_i
    
    results = Uzumibi::SQL.query(
      "SELECT * FROM users WHERE id = ?",
      [user_id]
    )
    
    if results.length > 0
      user = results[0]
      res.status_code = 200
      res.headers = { "Content-Type" => "application/json" }
      res.body = JSON.generate(user)
    else
      res.status_code = 404
      res.body = "User not found"
    end
    
    res
  end
  
  post "/users" do |req, res|
    data = JSON.parse(req.body)
    
    Uzumibi::SQL.execute(
      "INSERT INTO users (name, email) VALUES (?, ?)",
      [data["name"], data["email"]]
    )
    
    res.status_code = 201
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({ created: true })
    res
  end
end
```
