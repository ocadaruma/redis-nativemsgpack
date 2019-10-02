## Benchmark

Rough benchmark using ruby and 'redis' gem.

Environment: Intel(R) Core(TM) i7-7600U CPU @ 2.80GHz 24GB RAM

```
$ cd benchmark
$ bundle install --path=vendor/bundle 
$ bundle exec ruby throughput.rb
MSGPACK.UPSERTI64
Took total of: 4.347949127 s
Per iteration: 4.347949127e-05 s (0.04347949127 ms)

SADD
Took total of: 4.076891604 s
Per iteration: 4.076891604e-05 s (0.04076891604 ms)
```

memory usage:

```
redis-cli> memory usage msgpack:key
(integer) 1142837
redis-cli> memory usage set:key
(integer) 4673011
```
