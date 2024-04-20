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
        -e RUST_LOG=$RUST_LOG \
        -e RUST_BACKTRACE=$RUST_BACKTRACE \
        -e DATABASE_URL=postgres://magicdocs:magicdocs@magicdocs-db:5432/magicdocs \
        -e KEYCLOAK_INTERNAL_ADDR=http://magicdocs-keycloak:8080 \
        -e KEYCLOAK_EXTERNAL_ADDR=http://192.168.1.234:8080 \
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
    *)
        echo "Usage: $0 {start|stop|restart|build}"
        exit 1
        ;;
esac

