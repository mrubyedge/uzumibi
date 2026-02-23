# Error Handling

If a route is not found, Uzumibi automatically returns a 404 response:

```
Status: 404 Not Found
Body: "Not Found"
```

To handle errors in your route:

```ruby
post "/api/data" do |req, res|
  begin
    data = JSON.parse(req.body)
    # Process data...
    
    res.status_code = 200
    res.body = "Success"
  rescue JSON::ParserError => e
    res.status_code = 400
    res.body = "Invalid JSON: #{e.message}"
  rescue => e
    res.status_code = 500
    res.body = "Error: #{e.message}"
  end
  res
end
```
