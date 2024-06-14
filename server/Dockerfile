### BUILDER IMAGE ###
FROM rust:1.78.0-alpine3.19 as builder

# Install system dependencies
RUN apk update && apk add --no-cache \
    libc-dev \
    binaryen \
    openssl-dev \
    musl-dev \
    gcc

RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-leptos

# Create a new rust project
RUN USER=root cargo new --bin app
WORKDIR /app

# Copy source code
COPY . .

ENV RUSTFLAGS="-C target-feature=-crt-static"

# Run unit tests
RUN CARGO_BUILD_JOBS=$(nproc) cargo test --features "ssr"

# Build the project
RUN CARGO_BUILD_JOBS=$(nproc) cargo leptos build --release -vv

### RUNNER IMAGE ###
FROM alpine:3.19.1

# Install system dependencies
RUN apk update && apk add --no-cache \
    openssl \
    ca-certificates \
    libgcc

# Create a new user and group
RUN addgroup -g 1000 magicdocs && \
    adduser -D -s /bin/sh -u 1000 -G magicdocs magicdocs

WORKDIR /home/magicdocs/bin/

# Copy the built binary and static files
COPY --from=builder /app/target/release/magicdocs .
COPY --from=builder /app/target/site site
COPY --from=builder /app/Cargo.toml .

RUN chown magicdocs:magicdocs magicdocs
RUN chown -R magicdocs:magicdocs site
RUN chown magicdocs:magicdocs Cargo.toml

USER magicdocs

ENV LEPTOS_SITE_ROOT=./site

CMD ["./magicdocs"]