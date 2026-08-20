# Error Handling

When no route matches, Uzumibi constructs:

- status: 404
- content type: `text/plain; charset=utf-8`
- body: `Not Found`

Handle expected application errors inside the route and set a complete response:

~~~ruby
post "/items" do |req, res|
  if req.body.is_a?(Hash) && req.body["name"]
    res.return(
      201,
      { "content-type" => "application/json" },
      JSON.generate({ "created" => req.body["name"] })
    )
  else
    res.return(
      400,
      { "content-type" => "application/json" },
      JSON.generate({ "error" => "name is required" })
    )
  end
end
~~~

An unhandled Ruby or adapter error crosses the Wasm boundary as a runtime error. The exact HTTP response and logging behavior then depend on the platform host; Uzumibi does not currently provide a global Ruby error-handler DSL.
