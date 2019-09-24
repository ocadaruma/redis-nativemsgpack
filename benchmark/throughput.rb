require "redis"

ITERATIONS = 100000

REDIS_PORT = ENV["REDIS_PORT"] || "6379"

class Benchmarker
  def initialize(conn)
    @conn = conn
  end

  def bench_command(cmd, &block)
    start = Time.now

    ITERATIONS.times do
      block.yield(@conn)
    end

    puts cmd

    duration = Time.now - start
    puts "Took total of: #{duration} s"

    per_iteration = duration / ITERATIONS
    per_iteration_ms = per_iteration * 1000
    puts "Per iteration: #{per_iteration} s (#{per_iteration_ms} ms)"

    puts ""
  end

  def setup(&block)
    block.yield(@conn)
  end
end

bench = Benchmarker.new(Redis.new(port: REDIS_PORT.to_i))

## benchmark MSGPACK.UPSERTI64 and HSET
i = 0
bench.bench_command("MSGPACK.UPSERTI64") do |conn|
  resp = conn.send("MSGPACK.UPSERTI64", "msgpack:key", i)
  if resp != 0 && resp != 1
    raise "Unexpected response."
  end
  i += 1
end

i = 0
bench.bench_command("HSET") do |conn|
  resp = conn.send("HSET", "hset:key", i, "0")
  if resp != 0 && resp != 1
    raise "Unexpected response."
  end
  i += 1
end
