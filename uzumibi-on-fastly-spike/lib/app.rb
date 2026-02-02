class App < Uzumibi::Router
  get "/" do |req, res|
    debug_console("[Uzumibi] Received request at /")
    debug_console("[Uzumibi] Requested UA: #{req.headers["user-agent"]}")

    res.status_code = 200
    res.headers = {
      "Content-Type" => "text/plain",
      "X-Powered-By" => "#{RUBY_ENGINE} #{RUBY_VERSION}"
    }
    res.body = 
      "\"Uzumibi\" is a Japanese term that refers\n" +
      "to live embers buried under a layer of ash\n" +
      "to keep the fire from going out.\n"
    res
  end

  post "/data" do |req, res|
    debug_console("[Uzumibi] Received request at /data")
    debug_console("[Uzumibi] Body size: #{req.body.size} bytes")

    res.status_code = 200
    res.headers = {
      "Content-Type" => "text/plain",
      "X-Powered-By" => "#{RUBY_ENGINE} #{RUBY_VERSION}"
    }
    res.body = "Received data: #{req.params.inspect}\n"
    res
  end
end
$APP = App.new