# Error Handling

### Custom Error Responses

```ruby
class App < Uzumibi::Router
  get "/error-demo" do |req, res|
    error_type = req.params[:type]
    
    case error_type
    when "404"
      res.status_code = 404
      res.headers = { "Content-Type" => "application/json" }
      res.body = JSON.generate({
        error: "Not Found",
        message: "The requested resource was not found"
      })
    when "500"
      res.status_code = 500
      res.headers = { "Content-Type" => "application/json" }
      res.body = JSON.generate({
        error: "Internal Server Error",
        message: "Something went wrong"
      })
    when "401"
      res.status_code = 401
      res.headers = {
        "Content-Type" => "application/json",
        "WWW-Authenticate" => "Bearer"
      }
      res.body = JSON.generate({
        error: "Unauthorized",
        message: "Authentication required"
      })
    else
      res.status_code = 200
      res.body = "Specify ?type=404|500|401"
    end
    res
  end
  
  # Handle errors in route
  post "/api/data" do |req, res|
    begin
      data = JSON.parse(req.body)
      
      # Process data...
      
      res.status_code = 200
      res.headers = { "Content-Type" => "application/json" }
      res.body = JSON.generate({ success: true })
    rescue JSON::ParserError => e
      res.status_code = 400
      res.headers = { "Content-Type" => "application/json" }
      res.body = JSON.generate({
        error: "Bad Request",
        message: "Invalid JSON: #{e.message}"
      })
    rescue => e
      debug_console("[ERROR] #{e.message}")
      res.status_code = 500
      res.headers = { "Content-Type" => "application/json" }
      res.body = JSON.generate({
        error: "Internal Server Error",
        message: "An unexpected error occurred"
      })
    end
    res
  end
end

$APP = App.new
```
