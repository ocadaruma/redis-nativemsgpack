## Benchmark

Rough benchmark using ruby and 'redis' gem.

Environment: iMac 2019, 3.6GHz Corei9, 16GB DDR4

```
$ cd benchmark
$ bundle install --path=vendor/bundle 
$ bundle exec ruby throughput.rb
MSGPACK.UPSERTI64
Took total of: 3.733351 s
Per iteration: 3.733351e-05 s (0.03733351 ms)

HSET
Took total of: 3.691707 s
Per iteration: 3.6917070000000004e-05 s (0.03691707 ms)
```

memory usage:

```
redis-cli> memory usage msgpack:key
(integer) 1142835
redis-cli> memory usage hset:key
(integer) 4448722
```
