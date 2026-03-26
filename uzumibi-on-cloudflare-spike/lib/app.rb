class App < Uzumibi::Router
  Uzumibi::Access.team = "xxx-spike"

  get "/" do |req, res|
    debug_console("[Uzumibi] Received request at /")
    debug_console("[Uzumibi] Requested UA: #{req.headers["user-agent"]}")
    debug_console("[Uzumibi] Request Cookie: #{req.cookie.inspect}")

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

  get "/users/me" do |req, res|
    auth = req.cookie["CF_Authorization"]
    user = Uzumibi::Access.get_identity(auth)
    hash = {
      "email" => user.email,
      "id" => user.user_uuid,
      "data" => user.raw_data
    }
    debug_console("[Uzumibi] Authenticated user: #{hash.inspect}")

    res.return(
      200,
      { "Content-Type" => "application/json" },
      JSON.generate({ "email" => user.email })
    )
  end

  get "/rand/:seed" do |req, res|
    Random.srand(req.params[:seed].to_i) 

    res.status_code = 200
    res.headers = { "Content-Type" => "text/plain" }
    res.body = "Answer = #{rand(100)}\n"
    res
  end

  get "/yesno" do |req, res|
    remote_res = Uzumibi::Fetch.fetch("https://yesno.wtf/api", "GET", "")
    debug_console("[Uzumibi] Fetched from yesno.wtf: #{remote_res.status_code}")
    debug_console("[Uzumibi] headers: #{remote_res.headers.inspect}")

    res.status_code = 200
    res.headers = {
      "Content-Type" => "application/json",
    }
    res.body = remote_res.body
    res
  end

  get "/queue/send" do |req, res|
    Uzumibi::Queue.send("UZUMIBI_QUEUE", "Hello from Uzumibi Queue!")
    res.status_code = 200
    res.headers = { "Content-Type" => "text/plain" }
    res.body = "Sent message to queue\n"
    res
  end

  get "/assets/*" do |req, res|
    fetch_assets
  end

  get "/healthz" do |req, res|
    res.status_code = 200
    res.headers = {
      "Content-Type" => "application/json",
    }
    res.body = "{\"status\":\"ok\"}\n"
    res
  end
end

$APP = App.new