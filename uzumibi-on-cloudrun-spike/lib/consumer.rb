class Consumer < Uzumibi::Consumer
  def on_receive(message)
    debug_console("[Uzumibi::Queue] Received message id=#{message.id}, attempts=#{message.attempts}")
    message.ack!
  end
end

$CONSUMER = Consumer.new
