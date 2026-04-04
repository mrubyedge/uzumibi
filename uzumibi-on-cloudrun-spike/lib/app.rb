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

  get "/users/me" do |req, res|
    debug_console("[Uzumibi] Received request: #{req.headers.inspect}")
    auth = req.headers["x-goog-iap-jwt-assertion"]
    debug_console("[Uzumibi] Authenticated token length: #{auth&.size}")
    if auth.nil? || auth.empty?
      res.return(503, { "Content-Type" => "text/plain" }, "IAP Authentication token is missing\n")
    else
      user = Uzumibi::Access.get_identity(auth)
      hash = {
        "email" => user.email,
        "id" => user.user_uuid,
      }
      debug_console("[Uzumibi] Authenticated user: #{hash.inspect}")

      res.return(
        200,
        { "Content-Type" => "application/json" },
        JSON.generate({ "email" => user.email })
      )
    end
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
    Uzumibi::Queue.send("projects/#{Uzumibi::Google.project_id}/topics/uzumibi-spike", "Hello from Uzumibi Queue!")
    res.status_code = 200
    res.headers = { "Content-Type" => "text/plain" }
    res.body = "Sent message to queue\n"
    res
  end

  get "/firestore/get" do |req, res|
    value = Uzumibi::KV.get("hello")
    res.return(200, { "Content-Type" => "application/json" }, JSON.generate({
      "result" => value
    }))
  end

  get "/firestore/set" do |req, res|
    Uzumibi::KV.set("hello", "world")
    res.return(200, { "Content-Type" => "application/json" }, JSON.generate({
      "result" => "OK"
    }))
  end
end

$APP = App.new