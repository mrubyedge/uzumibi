# Editing Ruby Files

Open `lib/app.rb` and define your routes:

```ruby
class App < Uzumibi::Router
  get "/" do |req, res|
    res.status_code = 200
    res.headers = {
      "Content-Type" => "text/plain",
      "X-Powered-By" => "#{RUBY_ENGINE} #{RUBY_VERSION}"
    }
    res.body = "Hello from Uzumibi on the edge!\n"
    res
  end

  get "/hello/:name" do |req, res|
    res.status_code = 200
    res.headers = { "Content-Type" => "text/plain" }
    res.body = "Hello, #{req.params[:name]}!\n"
    res
  end

  post "/data" do |req, res|
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({ received: req.body })
    res
  end
end

$APP = App.new
```

The Ruby code is compiled to mruby bytecode during the build process and embedded into the WASM module.
