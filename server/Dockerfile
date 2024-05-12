### BUILDER IMAGE ###
FROM rust:1.77.2-alpine3.19 as builder

# Install system dependencies
RUN apk update && apk add --no-cache \
    openssl-dev \
    musl-dev \
    gcc \
    curl \
    nodejs \
    npm

# Create a new rust project
RUN USER=root cargo new --bin app
WORKDIR /app

# Download the tailwindcss binary for the target architecture
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


# Copy the dependencies files
COPY Cargo.toml Cargo.lock ./
COPY migration migration
COPY entity entity

ENV RUSTFLAGS="-C target-feature=-crt-static"

# Build the dependencies for release
RUN cargo build --release

# Remove artifacts from the previous build
RUN rm -f /app/target/release/deps/magicdocs*

# Copy the source code
COPY src src

# Run unit tests
RUN cargo test

# Build the project
RUN cargo build --release

# Copy the static files
COPY static static
COPY input.css .
COPY tailwind.config.js .
COPY templates templates

# Build the CSS
RUN ./tailwindcss -i input.css -o static/css/styles.css --minify

# Copy the typescript files
COPY typescript typescript

# Build the typescript files
RUN cd typescript && npm install && npm run build

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
COPY --from=builder /app/templates/ templates/
COPY --from=builder /app/static/ static/

RUN chown magicdocs:magicdocs magicdocs

USER magicdocs

CMD ["./magicdocs"]