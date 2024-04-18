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
    docker run -d --name magicdocs-keycloak \
        -p 8080:8080 \
        --network=magicdocs-net \
        -e KEYCLOAK_ADMIN=$KEYCLOAK_ADMIN \
        -e KEYCLOAK_ADMIN_PASSWORD=$KEYCLOAK_ADMIN_PASSWORD \
        -e KC_DB=postgres \
        -e KC_DB_USERNAME=$KC_DB_USERNAME \
        -e KC_DB_PASSWORD=$KC_DB_PASSWORD \
        -e KC_DB_URL_HOST=$KC_DB_URL_HOST \
        -e KC_DB_URL_PORT=$KC_DB_URL_PORT \
        -e KC_DB_URL_DATABASE=$KC_DB_URL_DATABASE \
        keycloak:latest start-dev --import-realm
}

function stop() {
    docker stop magicdocs-keycloak
    docker rm magicdocs-keycloak
}

function restart() {
    stop
    start
}

function build() {
    DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
    docker build -t keycloak:latest $DIR
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

