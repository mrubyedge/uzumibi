class Consumer < Uzumibi::Consumer
  # @rbs message: Uzumibi::Message
  def on_receive(message)
    debug_console("[Consumer] Received message: id=#{message.id}, body=#{message.body}, attempts=#{message.attempts}")

    if message.attempts > 5
      debug_console("[Consumer] Acknowledging message #{message.id} after 5 attempts!!")
      message.ack!
    else
      message.retry(delay_seconds: 10)
    end
  end
end

$CONSUMER = Consumer.new
