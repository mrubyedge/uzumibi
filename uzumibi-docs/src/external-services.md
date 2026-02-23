# External Service Abstractions

## What are External Service Abstractions?

External Service Abstractions are unified APIs that allow your Uzumibi application to access platform-specific services (like key-value stores, caches, databases, etc.) through a common interface. This abstraction layer enables you to write code once and deploy to multiple platforms without platform-specific modifications.

Each edge platform provides different services with different APIs:
- Cloudflare Workers has KV, R2, Durable Objects
- Fastly Compute has KV Store, Edge Dictionary
- Spin has Key-Value Store, SQLite
- Cloud Run can access Google Cloud services

External Service Abstractions provide a unified Ruby API that translates to the appropriate platform-specific implementation at runtime.

### Benefits

- **Write Once, Deploy Anywhere**: Same code works across platforms
- **Platform Independence**: Switch platforms without rewriting service access code
- **Consistent API**: Familiar Ruby interface regardless of underlying platform
- **Type Safety**: Well-defined interfaces reduce errors

### Status

**Current Status**: TBA (To Be Announced)

The External Service Abstractions layer is currently under development. The following sections describe the planned architecture and APIs.

## Available Services

### KV (Key-Value Store)

Key-value storage for simple data persistence.

**Status**: TBA

#### Planned API

```ruby
# Store a value
Uzumibi::KV.put("user:123", "John Doe")

# Retrieve a value
name = Uzumibi::KV.get("user:123")  # => "John Doe"

# Delete a value
Uzumibi::KV.delete("user:123")

# Check if key exists
exists = Uzumibi::KV.exists?("user:123")  # => true/false

# List keys with prefix
keys = Uzumibi::KV.list(prefix: "user:")  # => ["user:123", "user:456"]
```

#### Platform Mapping

| Platform | Implementation |
|----------|----------------|
| Cloudflare Workers | Workers KV |
| Fastly Compute | Fastly KV Store |
| Spin | Spin KV Store |
| Cloud Run | TBA |

### Cache

HTTP caching layer for response caching.

**Status**: TBA

#### Planned API

```ruby
# Store in cache
Uzumibi::Cache.put(
  "cache-key", 
  response_body,
  ttl: 3600  # seconds
)

# Retrieve from cache
cached = Uzumibi::Cache.get("cache-key")

# Delete from cache
Uzumibi::Cache.delete("cache-key")

# Clear all cache
Uzumibi::Cache.clear
```

#### Platform Mapping

| Platform | Implementation |
|----------|----------------|
| Cloudflare Workers | Cache API |
| Fastly Compute | Edge Cache |
| Spin | TBA |
| Cloud Run | TBA |

### Secret

Secure storage for API keys, tokens, and sensitive configuration.

**Status**: TBA

#### Planned API

```ruby
# Access secrets
api_key = Uzumibi::Secret.get("API_KEY")
db_password = Uzumibi::Secret.get("DB_PASSWORD")
```

#### Platform Mapping

| Platform | Implementation |
|----------|----------------|
| Cloudflare Workers | Environment Variables / Secrets |
| Fastly Compute | Secret Store |
| Spin | Spin Variables |
| Cloud Run | Secret Manager |

### ObjectStore

Object storage for files and binary data.

**Status**: TBA

#### Planned API

```ruby
# Upload object
Uzumibi::ObjectStore.put(
  "images/photo.jpg",
  image_data,
  content_type: "image/jpeg"
)

# Download object
image_data = Uzumibi::ObjectStore.get("images/photo.jpg")

# Delete object
Uzumibi::ObjectStore.delete("images/photo.jpg")

# List objects
objects = Uzumibi::ObjectStore.list(prefix: "images/")
```

#### Platform Mapping

| Platform | Implementation |
|----------|----------------|
| Cloudflare Workers | R2 |
| Fastly Compute | TBA |
| Spin | TBA |
| Cloud Run | Cloud Storage |

### Queue

Message queue for asynchronous task processing.

**Status**: TBA

#### Planned API

```ruby
# Send message to queue
Uzumibi::Queue.send(
  "notifications",
  { user_id: 123, message: "Hello" }
)

# Consume messages (in queue consumer handler)
Uzumibi::Queue.on_message do |message|
  # Process message
  user_id = message[:user_id]
  # ...
end
```

#### Platform Mapping

| Platform | Implementation |
|----------|----------------|
| Cloudflare Workers | Queue (Workers for Platforms) |
| Fastly Compute | TBA |
| Spin | TBA |
| Cloud Run | Cloud Tasks / Pub/Sub |

### SQL

SQL database access.

**Status**: TBA

#### Planned API

```ruby
# Execute query
results = Uzumibi::SQL.query(
  "SELECT * FROM users WHERE id = ?",
  [123]
)

# Execute update
affected = Uzumibi::SQL.execute(
  "UPDATE users SET name = ? WHERE id = ?",
  ["New Name", 123]
)

# Transaction support
Uzumibi::SQL.transaction do |tx|
  tx.execute("INSERT INTO users (name) VALUES (?)", ["Alice"])
  tx.execute("INSERT INTO logs (action) VALUES (?)", ["user_created"])
end
```

#### Platform Mapping

| Platform | Implementation |
|----------|----------------|
| Cloudflare Workers | D1 |
| Fastly Compute | TBA |
| Spin | SQLite |
| Cloud Run | Cloud SQL |

### Fetch

HTTP client for making requests to external APIs.

**Status**: TBA

#### Planned API

```ruby
# GET request
response = Uzumibi::Fetch.get("https://api.example.com/data")
data = JSON.parse(response.body)

# POST request
response = Uzumibi::Fetch.post(
  "https://api.example.com/users",
  body: JSON.generate({ name: "Alice" }),
  headers: {
    "Content-Type" => "application/json",
    "Authorization" => "Bearer #{token}"
  }
)

# Full request
response = Uzumibi::Fetch.request(
  method: "PUT",
  url: "https://api.example.com/resource",
  headers: { "Content-Type" => "application/json" },
  body: JSON.generate({ data: "value" })
)
```

#### Platform Mapping

| Platform | Implementation |
|----------|----------------|
| Cloudflare Workers | fetch() API |
| Fastly Compute | Backend requests |
| Spin | Outbound HTTP |
| Cloud Run | HTTP client |

### Others

Additional services being considered:

- **Analytics**: Request analytics and metrics
- **Logging**: Structured logging
- **Tracing**: Distributed tracing
- **Email**: Email sending
- **Websockets**: WebSocket connections (platform-dependent)

## Platform Support Matrix

| Service | Cloudflare | Fastly | Spin | Cloud Run |
|---------|-----------|--------|------|-----------|
| **KV** | ✅ Workers KV | ✅ KV Store | ✅ KV Store | ❌ TBA |
| **Cache** | ✅ Cache API | ✅ Edge Cache | ❌ TBA | ❌ TBA |
| **Secret** | ✅ Secrets | ✅ Secret Store | ✅ Variables | ✅ Secret Manager |
| **ObjectStore** | ✅ R2 | ❌ TBA | ❌ TBA | ✅ Cloud Storage |
| **Queue** | ✅ Queues | ❌ TBA | ❌ TBA | ✅ Pub/Sub |
| **SQL** | ✅ D1 | ❌ TBA | ✅ SQLite | ✅ Cloud SQL |
| **Fetch** | ✅ fetch API | ✅ Backends | ✅ Outbound HTTP | ✅ HTTP |

Legend:
- ✅ Planned/Available
- ❌ Not Available/TBA
- TBA: To Be Announced

## Usage Examples

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

## Development Roadmap

The External Service Abstractions are being developed in phases:

### Phase 1: Core Services
- KV Store
- Fetch (HTTP Client)
- Secret Management

### Phase 2: Storage
- Cache
- Object Store

### Phase 3: Advanced Services
- SQL Database
- Queue
- Logging & Analytics

### Phase 4: Platform-Specific Features
- Platform-specific optimizations
- Advanced features

## Contributing

The External Service Abstractions layer is under active development. If you're interested in contributing or have suggestions for the API design, please:

1. Check the GitHub repository for current status
2. Open issues for API suggestions
3. Submit PRs for implementations

## Next Steps

- Review the [Ruby API Reference](./ruby-api.md) for current stable APIs
- Check [Supported Platforms](./platforms.md) for platform-specific details
- See [Examples](./examples.md) for working code samples
