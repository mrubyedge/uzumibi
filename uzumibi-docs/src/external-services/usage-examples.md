# Cloudflare Usage Examples

These examples require a project generated with `--features enable-external` unless noted otherwise.

## KV-backed counter

~~~ruby
class App < Uzumibi::Router
  get "/counter" do |req, res|
    count = (Uzumibi::KV.get("counter") || "0").to_i
    res.return(
      200,
      { "content-type" => "application/json" },
      JSON.generate({ "count" => count })
    )
  end

  post "/counter/increment" do |req, res|
    count = (Uzumibi::KV.get("counter") || "0").to_i + 1
    Uzumibi::KV.set("counter", count.to_s)
    res.return(
      200,
      { "content-type" => "application/json" },
      JSON.generate({ "count" => count })
    )
  end
end

$APP = App.new
~~~

Configure `UZUMIBI_KV` in `wrangler.jsonc` before running this application.

## Outbound JSON request

~~~ruby
get "/upstream" do |req, res|
  upstream = Uzumibi::Fetch.fetch(
    "https://example.com/api",
    "GET",
    "",
    { "accept" => "application/json" }
  )

  res.return(
    upstream.status_code,
    { "content-type" => upstream.headers["content-type"] || "text/plain" },
    upstream.body
  )
end
~~~

## Send a Queue message

After configuring a producer binding named `UZUMIBI_QUEUE`:

~~~ruby
post "/jobs" do |req, res|
  Uzumibi::Queue.send("UZUMIBI_QUEUE", req.raw_body)
  res.return(202, { "content-type" => "text/plain" }, "queued\n")
end
~~~

## Queue consumer

This example belongs in `lib/consumer.rb` of a project generated with `--features queue`:

~~~ruby
class Consumer < Uzumibi::Consumer
  def on_receive(message)
    begin
      payload = JSON.parse(message.body)
      debug_console("processing #{payload.inspect}")
      message.ack!
    rescue => error
      debug_console("failed: #{error.message}")
      message.retry(delay_seconds: 10)
    end
  end
end

$CONSUMER = Consumer.new
~~~
