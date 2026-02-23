# Helper Functions

### `debug_console(message)`

Output debug messages to the console:

```ruby
get "/debug" do |req, res|
  debug_console("[Debug] Request received: #{req.path}")
  debug_console("[Debug] Headers: #{req.headers.inspect}")
  
  res.status_code = 200
  res.body = "Check console for debug output"
  res
end
```

Note: The exact behavior of `debug_console` depends on the platform:
- Cloudflare Workers: Outputs to Workers console
- Fastly Compute: Outputs to Fastly logs
- Local development: Outputs to stdout
