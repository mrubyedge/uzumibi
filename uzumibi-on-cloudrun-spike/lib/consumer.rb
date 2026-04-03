class Consumer < Uzumibi::Consumer
  def on_receive(message)
    debug_console("[Uzumibi::Queue] Received message id=#{message.id}")
    debug_console("[Uzumibi::Queue] Message data: #{message.body.inspect}")
    if message.attempts < 3
      debug_console("[Uzumibi::Queue] Simulating failure for message id=#{message.id}")
      raise "Simulated processing failure for message id=#{message.id}"
      message.retry(delay_seconds: 3)
    else
      debug_console("[Uzumibi::Queue] Processing succeeded for message id=#{message.id} after #{message.attempts} attempts")
      message.ack!
    end
  end
end

$CONSUMER = Consumer.new
