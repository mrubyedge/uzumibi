# Available Services

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
