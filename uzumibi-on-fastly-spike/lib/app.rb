class App < Uzumibi::Router
  get "/" do |req, res|
    debug_console("[Uzumibi] Received request at /")
    debug_console("[Uzumibi] Requested UA: #{req.headers["user-agent"]}")

    res.status_code = 200
    res.headers = {
      "Content-Type" => "text/plain",
      "X-Powered-By" => "#{RUBY_ENGINE} #{RUBY_VERSION}"
    }
    res.body = "Hello, this is Uzumibi, powered by #{RUBY_ENGINE} - #{RUBY_VERSION}\n"
    res
  end
end

$APP = App.new