#-------------------------------------------------------------------------------
# Build module
#-------------------------------------------------------------------------------
FROM rust:1.38.0 AS builder

WORKDIR /build

# cache dependencies
COPY ./Cargo.toml ./Cargo.toml
COPY ./src/lib.rs ./src/lib.rs
RUN cargo fetch

# build
COPY . ./
RUN cargo build --release

#-------------------------------------------------------------------------------
# Redis
#-------------------------------------------------------------------------------
FROM redis:5.0.6

RUN mkdir -p /etc/redis/modules/
COPY docker/redis.conf /etc/redis/redis.conf
COPY --from=builder /build/target/release/libredis_nativemsgpack.so /etc/redis/modules/libredis_nativemsgpack.so
CMD [ "redis-server", "/etc/redis/redis.conf"]