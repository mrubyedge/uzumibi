class App < Uzumibi::Router
  get "/" do |req, res|
    res.status_code = 200
    res.headers = {
      "Content-Type" => "application/json",
      "X-Powered-By" => "serviceworker-spike"
    }
    res.body = JSON.stringify({
      message: "Hello from Service Worker!",
      timestamp: Time.now.to_s,
      path: "/",
      description: "This is the root endpoint",
      engine: "#{RUBY_ENGINE} #{RUBY_VERSION}"
    })
    res
  end

  get "/hello" do |req, res|
    res.status_code = 200
    res.headers = {
      "Content-Type" => "application/json",
      "X-Powered-By" => "serviceworker-spike"
    }
    res.body = JSON.stringify({
      message: "Hello, World!",
      timestamp: Time.now.to_s,
      path: "/hello",
      description: "This is the hello endpoint",
      greeting: "Welcome to the hello page!",
      engine: "#{RUBY_ENGINE} #{RUBY_VERSION}"
    })
    res
  end

  get "/profile" do |req, res|
    res.status_code = 200
    res.headers = {
      "Content-Type" => "application/json",
      "X-Powered-By" => "serviceworker-spike"
    }
    res.body = JSON.stringify({
      message: "User Profile",
      timestamp: Time.now.to_s,
      path: "/profile",
      description: "This is the profile endpoint",
      user: {
        id: 12345,
        name: "Sample User",
        email: "user@example.com",
        role: "developer"
      },
      engine: "#{RUBY_ENGINE} #{RUBY_VERSION}"
    })
    res
  end
end

$APP = App.new
