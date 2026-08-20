# Editing Ruby Files

For an HTTP application, edit `lib/app.rb`:

~~~ruby
class App < Uzumibi::Router
  get "/" do |req, res|
    res.return(
      200,
      { "content-type" => "text/plain" },
      "Hello from Uzumibi!\n"
    )
  end

  post "/echo/:name" do |req, res|
    res.status_code = 200
    res.headers = { "content-type" => "text/plain" }
    res.body = "#{req.params[:name]}: #{req.raw_body}\n"
    res
  end
end

$APP = App.new
~~~

Each route must set the response status, headers, and body. Ending with `res` is the conventional style. `res.return(status, headers, body)` is a convenience method that sets all three fields and returns `res`.

For a Queue consumer generated with `--features queue`, edit `lib/consumer.rb` and keep the generated `$CONSUMER` global.

Ruby source is compiled to mruby bytecode during the build. The generated development command rebuilds it automatically when the command is restarted.
