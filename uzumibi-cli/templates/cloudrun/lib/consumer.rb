class Consumer < Uzumibi::Consumer
  # @rbs message: Uzumibi::Message
  def on_receive(message)
    debug_console("[Consumer] Received message: id=#{message.id}, body=#{message.body}, attempts=#{message.attempts}")
    message.ack!
  end
end

$CONSUMER = Consumer.new
