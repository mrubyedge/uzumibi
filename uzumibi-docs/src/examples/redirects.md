# Redirects

### Redirect Examples

```ruby
class App < Uzumibi::Router
  # Temporary redirect (302)
  get "/old-path" do |req, res|
    res.status_code = 302
    res.headers = {
      "Location" => "/new-path",
      "Content-Type" => "text/plain"
    }
    res.body = "Redirecting..."
    res
  end
  
  # Permanent redirect (301)
  get "/moved" do |req, res|
    res.status_code = 301
    res.headers = {
      "Location" => "/permanently-moved",
      "Content-Type" => "text/plain"
    }
    res.body = "Moved Permanently"
    res
  end
  
  # Redirect with parameters
  get "/user/:id" do |req, res|
    user_id = req.params[:id]
    res.status_code = 302
    res.headers = { "Location" => "/users/#{user_id}/profile" }
    res.body = ""
    res
  end
end

$APP = App.new
```
