# Platform Helper Functions

Helper functions are supplied by platform adapters, not by the core `uzumibi-gem`.

## `debug_console(message)`

The Cloudflare adapter converts the argument with `to_s` and writes it through the Worker console:

~~~ruby
get "/debug" do |req, res|
  debug_console("request path: #{req.path}")
  res.return(200, {}, "logged\n")
end
~~~

Other templates can map the same helper to their own logging facility. Logging destination and behavior are platform-specific.

## `fetch_assets`

In a Cloudflare HTTP application, `fetch_assets` stops Ruby request handling and delegates the original request to the `ASSETS` binding:

~~~ruby
get "/assets/*" do |req, res|
  fetch_assets
end
~~~

This helper is Cloudflare-specific.
