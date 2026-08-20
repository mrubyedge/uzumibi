# Routing

Define routes as class methods on a subclass of `Uzumibi::Router`.

~~~ruby
class App < Uzumibi::Router
  get "/items" do |req, res|
    res.return(200, { "content-type" => "text/plain" }, "items\n")
  end

  post "/items" do |req, res|
    res.return(201, { "content-type" => "text/plain" }, "created\n")
  end
end

$APP = App.new
~~~

The router supports `get`, `post`, `put`, `delete`, `head`, and `options`.

## Named parameters

~~~ruby
get "/users/:user_id/posts/:post_id" do |req, res|
  user_id = req.params[:user_id]
  post_id = req.params[:post_id]
  res.return(200, {}, "#{user_id}/#{post_id}")
end
~~~

## Wildcards

A trailing `*` captures the remaining path in `req.params[:"*"]`:

~~~ruby
get "/assets/*" do |req, res|
  path = req.params[:"*"]
  res.return(200, {}, path)
end
~~~

## Query parameters

Query parameters are merged into `req.params` as Symbol keys:

~~~ruby
get "/search" do |req, res|
  query = req.params[:q]
  res.return(200, {}, query || "")
end
~~~

The current query parser is intentionally small: it splits `&` and `=` pairs and does not URL-decode them. Form-urlencoded request bodies use a separate percent-decoding parser.

## HEAD and missing routes

A HEAD request uses the GET router for the same path and clears the response body after the handler runs. If no method/path pair matches, Uzumibi returns status 404 with body `Not Found`.
