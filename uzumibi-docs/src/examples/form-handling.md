# Form Handling

### Processing Form Data

Handle form submissions:

```ruby
class App < Uzumibi::Router
  # Show form (HTML)
  get "/form" do |req, res|
    res.status_code = 200
    res.headers = { "Content-Type" => "text/html" }
    res.body = <<~HTML
      <!DOCTYPE html>
      <html>
      <head><title>Form Example</title></head>
      <body>
        <h1>Submit Form</h1>
        <form method="POST" action="/form">
          <label>Name: <input type="text" name="name"></label><br>
          <label>Email: <input type="email" name="email"></label><br>
          <button type="submit">Submit</button>
        </form>
      </body>
      </html>
    HTML
    res
  end
  
  # Process form submission
  post "/form" do |req, res|
    # Form data is automatically parsed into req.params
    # when Content-Type is application/x-www-form-urlencoded
    name = req.params[:name]
    email = req.params[:email]
    
    res.status_code = 200
    res.headers = { "Content-Type" => "text/html" }
    res.body = <<~HTML
      <!DOCTYPE html>
      <html>
      <head><title>Form Submitted</title></head>
      <body>
        <h1>Thank you!</h1>
        <p>Name: #{name}</p>
        <p>Email: #{email}</p>
      </body>
      </html>
    HTML
    res
  end
end

$APP = App.new
```
