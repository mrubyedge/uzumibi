# Introduction

Welcome to the Uzumibi documentation!

Uzumibi is a lightweight web application framework for embedding MRuby into edge computing platforms like Cloudflare Workers, Fastly Compute@Edge, Spin, and more. It allows developers to write serverless applications using Ruby, leveraging the power of MRuby for efficient execution in constrained environments.

## Quick Example

Here's a simple example of an Uzumibi application:

```ruby
class App < Uzumibi::Router
  get "/" do |req, res|
    res.status_code = 200
    res.headers = {
      "content-type" => "text/plain",
      "x-powered-by" => "#{RUBY_ENGINE} #{RUBY_VERSION}"
    }
    res.body = "Hello from Uzumibi!"
    res
  end

  get "/greet/:name" do |req, res|
    res.status_code = 200
    res.headers = { "content-type" => "text/plain" }
    res.body = "Hello, #{req.params[:name]}!"
    res
  end
end

$APP = App.new
```

## Why Uzumibi?

- **Ruby on the Edge**: Write edge functions in Ruby instead of JavaScript
- **Lightweight**: Built on mruby/edge, optimized for WebAssembly and constrained environments
- **Multi-platform**: Deploy to Cloudflare Workers, Fastly Compute, Spin, and more
- **Simple API**: Familiar Sinatra-like routing DSL

## Get Started

Head over to the [Installation and Getting Started](./installation.md) guide to begin building with Uzumibi!
