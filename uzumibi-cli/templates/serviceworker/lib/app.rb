class App < Uzumibi::Router
  get "/" do |req, res|
    res.status_code = 200
    res.headers = {
      "Content-Type" => "text/plain",
      "X-Powered-By" => "#{RUBY_ENGINE} #{RUBY_VERSION}"
    }
    res.body = "It works!\n"
    res
  end
end

$APP = App.new
