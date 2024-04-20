# Builder image
FROM rust:1.77.2-alpine3.19 as builder

RUN apk update && apk add --no-cache \
    openssl-dev \
    musl-dev \
    gcc

# RUN rustup target add x86_64-unknown-linux-musl

RUN USER=root cargo new --bin app
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY migration migration

RUN RUSTFLAGS="-C target-feature=-crt-static" cargo build --release

RUN rm -f /app/target/release/deps/magicdocs*

COPY . .

RUN RUSTFLAGS="-C target-feature=-crt-static" cargo build --release


# Runner image
FROM alpine:3.19.1

RUN apk update && apk add --no-cache \
    openssl \
    ca-certificates \
    libgcc

RUN addgroup -g 1000 magicdocs && \
    adduser -D -s /bin/sh -u 1000 -G magicdocs magicdocs

WORKDIR /home/magicdocs/bin/

COPY --from=builder /app/target/release/magicdocs .
COPY --from=builder /app/templates/ templates/

RUN chown magicdocs:magicdocs magicdocs

USER magicdocs

CMD ["./magicdocs"]