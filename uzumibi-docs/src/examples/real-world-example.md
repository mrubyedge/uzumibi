# Real-World Example

### Complete Blog API

A more complete example showing a blog API:

```ruby
class App < Uzumibi::Router
  # Root endpoint
  get "/" do |req, res|
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      name: "Blog API",
      version: "1.0",
      endpoints: {
        posts: "/api/posts",
        authors: "/api/authors"
      }
    })
    res
  end
  
  # List posts
  get "/api/posts" do |req, res|
    page = (req.params[:page] || "1").to_i
    tag = req.params[:tag]
    
    posts = [
      { id: 1, title: "First Post", author_id: 1, tags: ["ruby", "web"] },
      { id: 2, title: "Second Post", author_id: 2, tags: ["edge", "wasm"] }
    ]
    
    # Filter by tag if provided
    posts = posts.select { |p| p[:tags].include?(tag) } if tag
    
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      page: page,
      posts: posts
    })
    res
  end
  
  # Get single post
  get "/api/posts/:id" do |req, res|
    post_id = req.params[:id].to_i
    
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      id: post_id,
      title: "Post #{post_id}",
      content: "Content for post #{post_id}",
      author_id: 1,
      created_at: Time.now.to_i
    })
    res
  end
  
  # Create post
  post "/api/posts" do |req, res|
    begin
      data = JSON.parse(req.body)
      
      if !data["title"] || data["title"].empty?
        res.status_code = 400
        res.body = JSON.generate({ error: "Title is required" })
      else
        new_id = rand(1000)
        res.status_code = 201
        res.headers = {
          "Content-Type" => "application/json",
          "Location" => "/api/posts/#{new_id}"
        }
        res.body = JSON.generate({
          id: new_id,
          title: data["title"],
          content: data["content"],
          author_id: data["author_id"],
          created_at: Time.now.to_i
        })
      end
    rescue JSON::ParserError
      res.status_code = 400
      res.body = JSON.generate({ error: "Invalid JSON" })
    end
    res
  end
  
  # List authors
  get "/api/authors" do |req, res|
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      authors: [
        { id: 1, name: "Alice", email: "alice@example.com" },
        { id: 2, name: "Bob", email: "bob@example.com" }
      ]
    })
    res
  end
  
  # Get single author
  get "/api/authors/:id" do |req, res|
    author_id = req.params[:id].to_i
    
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({
      id: author_id,
      name: "Author #{author_id}",
      email: "author#{author_id}@example.com",
      bio: "Biography for author #{author_id}"
    })
    res
  end
end

$APP = App.new
```
