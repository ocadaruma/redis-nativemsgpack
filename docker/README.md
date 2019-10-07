# redis-nativemsgpack docker image

This docker image is for trying it out redis-nativemsgpack.

## How to execute

Start server:

```sh
$ cd docker
$ docker-compose up -d
```

Run commands:

```sh
$ docker-compose exec redis redis-cli
127.0.0.1:6379> MSGPACK.UPSERTI64 key 11 7 5 3 2 -2
(integer) 1
127.0.0.1:6379> EVAL 'return cmsgpack.unpack(redis.call("GET", KEYS[1]))' 1 key
1) (integer) -2
2) (integer) 2
3) (integer) 3
4) (integer) 5
5) (integer) 7
6) (integer) 11
127.0.0.1:6379> MSGPACK.DELI64 key 7 -2 9999
(integer) 2
127.0.0.1:6379> EVAL 'return cmsgpack.unpack(redis.call("GET", KEYS[1]))' 1 key
1) (integer) 2
2) (integer) 3
3) (integer) 5
4) (integer) 11
```
