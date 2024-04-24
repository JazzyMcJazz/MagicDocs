#!/bin/bash
set -e

OP=$1

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
ENV_FILE="$DIR/.env"


# Check if the .env file exists
if [ ! -f "$ENV_FILE" ]; then
    echo "Error: .env file does not exist."
    exit 1
fi

# Load variables from .env file into local environment
while IFS='=' read -r key value
do
    if [[ "$key" =~ ^[A-Z0-9_]+$ ]]; then
        # Use eval to correctly handle value and avoid creating unintended variables
        declare "$key=$value"
    fi
done < "$ENV_FILE"

function start() {
    docker run -d --name magicdocs-server \
        -p 3000:3000 \
        --network=magicdocs-net \
        -e RUST_ENV=test \
        -e RUST_LOG=$RUST_LOG \
        -e RUST_BACKTRACE=$RUST_BACKTRACE \
        -e DATABASE_URL=postgres://magicdocs:magicdocs@magicdocs-db:5432/magicdocs \
        -e KEYCLOAK_INTERNAL_ADDR=$KEYCLOAK_INTERNAL_ADDR \
        -e KEYCLOAK_EXTERNAL_ADDR=$KEYCLOAK_EXTERNAL_ADDR \
        -e KEYCLOAK_USER=$KEYCLOAK_USER \
        -e KEYCLOAK_PASSWORD=$KEYCLOAK_PASSWORD \
        -e KEYCLOAK_REALM=$KEYCLOAK_REALM \
        -e KEYCLOAK_CLIENT=$KEYCLOAK_CLIENT \
        -e KEYCLOAK_CLIENT_SECRET=$KEYCLOAK_CLIENT_SECRET \
        magicdocs:latest
}

function stop() {
    docker stop magicdocs-server
    docker rm magicdocs-server
}

function restart() {
    stop
    start
}

function build() {
    DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
    docker build -t magicdocs:latest $DIR
}

function generate_entities() {
    sea generate entity -o $DIR/entity/src -l --with-serde serialize
}

function download_tailwind() {
    curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64
    chmod +x tailwindcss-linux-x64
}

function install_tailwind() {
    mv tailwindcss-linux-x64 /usr/local/sbin/tailwind
}

function run_tailwind() {
    tailwind -i input.css -o static/css/styles.css --watch
}

case $OP in
    start)
        start
        ;;
    stop)
        stop
        ;;
    restart)
        restart
        ;;
    build)
        build
        ;;
    generate)
        generate_entities
        ;;
    download_tailwind)
        download_tailwind
        ;;
    install_tailwind)
        install_tailwind
        ;;
    tailwind)
        run_tailwind
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|build}"
        exit 1
        ;;
esac

