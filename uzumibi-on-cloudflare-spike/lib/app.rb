class App < Uzumibi::Router
  get "/" do |req, res|
    debug_console("[Uzumibi] Received request at /")

    res.status_code = 200
    res.headers = { "Content-Type" => "text/plain" }
    res.body = "Hello, Uzumibi on Cloudflare Workers!"
    res
  end
end

$APP = App.new