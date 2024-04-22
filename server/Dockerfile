# Builder image
FROM rust:1.77.2-alpine3.19 as builder

RUN apk update && apk add --no-cache \
    openssl-dev \
    musl-dev \
    gcc \
    curl

RUN USER=root cargo new --bin app
WORKDIR /app

ARG ARCH
RUN if [ -z "$ARCH" ]; then \
        ARCH=$(uname -m); \
        case "$ARCH" in \
            x86_64) ARCH='x64' ;; \
            aarch64) ARCH='arm64' ;; \
            armv7l) ARCH='armv7' ;; \
            *) echo "Unsupported architecture: $ARCH"; exit 1 ;; \
        esac; \
    fi && \
    echo "Building for $ARCH" && \
    curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-${ARCH} && \
    chmod +x tailwindcss-linux-${ARCH} && \
    mv tailwindcss-linux-${ARCH} tailwindcss



COPY Cargo.toml Cargo.lock ./
COPY migration migration
COPY entity entity

RUN RUSTFLAGS="-C target-feature=-crt-static" cargo build --release

RUN rm -f /app/target/release/deps/magicdocs*

COPY . .

RUN RUSTFLAGS="-C target-feature=-crt-static" cargo build --release

RUN ./tailwindcss -i input.css -o static/css/styles.css --minify


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
COPY --from=builder /app/static/ static/

RUN chown magicdocs:magicdocs magicdocs

USER magicdocs

CMD ["./magicdocs"]