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
    docker run -d --name magicdocs-db \
        -p 5432:5432 \
        --network=magicdocs-net \
        --network-alias=db \
        -v magicdocs_db:/var/lib/postgresql/data \
        -e POSTGRES_PASSWORD=$PG_PASS \
        -e KC_DB_USER=$KC_DB_USER \
        -e KC_DB_PASS=$KC_DB_PASS \
        -e MD_DB_USER=$MD_DB_USER \
        -e MD_DB_PASS=$MD_DB_PASS \
        --health-cmd='pg_isready -U postgres -d keycloak && pg_isready -U postgres -d magicdocs' \
        --health-start-period=10s \
        --health-start-interval=5s \
        --health-interval=5m \
        --health-timeout=10s \
        --health-retries=3 \
        pgvector:latest
}

function stop() {
    docker stop magicdocs-db
    docker rm magicdocs-db
}

function restart() {
    stop
    start
}

function build() {
    docker build -t pgvector:latest $DIR
}

function network() {
    docker network create magicdocs-net
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
    network)
        network
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|build|network}"
        exit 1
        ;;
esac

