# HTTP Methods

### GET, POST, PUT, DELETE

Handle different HTTP methods:

```ruby
class App < Uzumibi::Router
  # GET - Retrieve resource
  get "/users/:id" do |req, res|
    user_id = req.params[:id]
    
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      id: user_id,
      name: "User #{user_id}",
      email: "user#{user_id}@example.com"
    })
    res
  end
  
  # POST - Create resource
  post "/users" do |req, res|
    data = JSON.parse(req.body)
    
    res.status_code = 201
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      id: rand(1000),
      name: data["name"],
      email: data["email"],
      created: true
    })
    res
  end
  
  # PUT - Update resource
  put "/users/:id" do |req, res|
    user_id = req.params[:id]
    data = JSON.parse(req.body)
    
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      id: user_id,
      name: data["name"],
      updated: true
    })
    res
  end
  
  # DELETE - Delete resource
  delete "/users/:id" do |req, res|
    user_id = req.params[:id]
    
    res.status_code = 204
    res.body = ""
    res
  end
end

$APP = App.new
```
