# redis-nativemsgpack

[![Build Status](https://travis-ci.org/ocadaruma/redis-nativemsgpack.svg?branch=master)](https://travis-ci.org/ocadaruma/redis-nativemsgpack)

A Redis module provides space-efficient set operations using msgpack encoding.

## Installation

1. Download and extract binary from [Releases](https://github.com/ocadaruma/redis-nativemsgpack/releases).
2. Load module.

```
redis-cli> MODULE LOAD /path/to/libredis_nativemsgpack.so
```

### Build

You can build manually if necessary.

```
$ git clone https://github.com/ocadaruma/redis-nativemsgpack.git
$ cd redis-nativemsgpack
$ cargo build --release
$ cp target/release/libredis_nativemsgpack.so /path/to/modules/
```

## Usage

### MSGPACK.UPSERTI64

```
redis-cli> MSGPACK.UPSERTI64 key 11 7 5 3 2 -2
(integer) 1
redis-cli> EVAL 'return cmsgpack.unpack(redis.call("GET", KEYS[1]))' 1 key
1) (integer) -2
2) (integer) 2
3) (integer) 3
4) (integer) 5
5) (integer) 7
6) (integer) 11
```

### MSGPACK.DELI64

```
redis-cli> MSGPACK.DELI64 key 7 -2 9999
(integer) 2
redis-cli> EVAL 'return cmsgpack.unpack(redis.call("GET", KEYS[1]))' 1 key
1) (integer) 2
2) (integer) 3
3) (integer) 5
4) (integer) 11
```

## Memory usage

Compact than Redis Sets data type.

```
$ for i in `seq 10000`; do redis-cli -p 6380 MSGPACK.UPSERTI64 msgpack:key $i > /dev/null; done
$ for i in `seq 10000`; do redis-cli -p 6380 SADD set:key $i > /dev/null; done
redis-cli> MEMORY USAGE set:key
(integer) 494753
redis-cli> MEMORY USAGE msgpack:key
(integer) 142899
```

## Performance

`UPSERTI64` performs almost as fast as `SADD`.

See results in [rough benchmark](benchmark/README.md).
