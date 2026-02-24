class App < Uzumibi::Router
  get "/" do |req, res|
    res.status_code = 200
    res.headers = {
      "Content-Type" => "application/json",
      "X-Powered-By" => "webworker-spike"
    }
    res.body = JSON.dump({
      message: "Hello from Web Worker!",
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
      "X-Powered-By" => "webworker-spike"
    }
    res.body = JSON.dump({
      message: "Hello, World!",
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
      "X-Powered-By" => "webworker-spike"
    }
    res.body = JSON.dump({
      message: "User Profile",
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

  get "/error" do |req, res|
    raise "This is a simulated error for testing error handling in the web worker environment."
  end
end

$APP = App.new
