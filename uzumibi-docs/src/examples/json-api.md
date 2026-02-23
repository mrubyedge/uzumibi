# JSON API

### RESTful API Example

A complete RESTful API example:

```ruby
class App < Uzumibi::Router
  # List all items
  get "/api/items" do |req, res|
    page = (req.params[:page] || "1").to_i
    
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      page: page,
      items: [
        { id: 1, name: "Item 1" },
        { id: 2, name: "Item 2" }
      ]
    })
    res
  end
  
  # Get single item
  get "/api/items/:id" do |req, res|
    item_id = req.params[:id].to_i
    
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      id: item_id,
      name: "Item #{item_id}",
      description: "Description for item #{item_id}"
    })
    res
  end
  
  # Create item
  post "/api/items" do |req, res|
    begin
      data = JSON.parse(req.body)
      
      # Validate
      if !data["name"] || data["name"].empty?
        res.status_code = 400
        res.body = JSON.generate({ error: "Name is required" })
      else
        res.status_code = 201
        res.headers = {
          "Content-Type" => "application/json",
          "Location" => "/api/items/#{rand(1000)}"
        }
        res.body = JSON.generate({
          id: rand(1000),
          name: data["name"],
          description: data["description"]
        })
      end
    rescue JSON::ParserError
      res.status_code = 400
      res.body = JSON.generate({ error: "Invalid JSON" })
    end
    res
  end
  
  # Update item
  put "/api/items/:id" do |req, res|
    item_id = req.params[:id].to_i
    
    begin
      data = JSON.parse(req.body)
      
      res.status_code = 200
      res.headers = { "Content-Type" => "application/json" }
      res.body = JSON.generate({
        id: item_id,
        name: data["name"],
        description: data["description"],
        updated: true
      })
    rescue JSON::ParserError
      res.status_code = 400
      res.body = JSON.generate({ error: "Invalid JSON" })
    end
    res
  end
  
  # Delete item
  delete "/api/items/:id" do |req, res|
    res.status_code = 204
    res.body = ""
    res
  end
end

$APP = App.new
```
