# Complete Example

~~~ruby
class App < Uzumibi::Router
  get "/" do |req, res|
    res.return(
      200,
      { "content-type" => "text/plain" },
      "Welcome to Uzumibi!\n"
    )
  end

  get "/users/:id" do |req, res|
    res.return(
      200,
      { "content-type" => "application/json" },
      JSON.generate({
        "id" => req.params[:id],
        "verbose" => req.params[:verbose]
      })
    )
  end

  post "/echo" do |req, res|
    res.return(
      200,
      { "content-type" => "application/octet-stream" },
      req.raw_body
    )
  end

  get "/old-path" do |req, res|
    res.return(
      302,
      { "location" => "/", "content-type" => "text/plain" },
      "Moved\n"
    )
  end
end

$APP = App.new
~~~

The `application/octet-stream` example describes the core Ruby response. Check the selected platform adapter before relying on arbitrary binary response bytes; the current Cloudflare host text-decodes response bodies.
