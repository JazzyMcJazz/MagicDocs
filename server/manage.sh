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
        -e MY_LOG=$MY_LOG \
        -e RUST_BACKTRACE=$RUST_BACKTRACE \
        -e DATABASE_URL=postgres://magicdocs:magicdocs@magicdocs-db:5432/magicdocs \
        -e KEYCLOAK_URL=$KEYCLOAK_URL \
        -e KEYCLOAK_REALM=$KEYCLOAK_REALM \
        -e KEYCLOAK_CLIENT_NAME=$KEYCLOAK_CLIENT_NAME \
        -e KEYCLOAK_CLIENT_UUID=$KEYCLOAK_CLIENT_UUID \
        -e KEYCLOAK_CLIENT_SECRET=$KEYCLOAK_CLIENT_SECRET \
        -e OPENAI_API_KEY=$OPENAI_API_KEY \
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

    # Replace the generated code lines with custom code
    sed -i 's/ Eq, / /g' $DIR/entity/src/embedding.rs
    sed -i 's/pub embedding: String,/pub embedding: Vec<f32>,/g' $DIR/entity/src/embedding.rs
}

function download_tailwind() {
    curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64
    chmod +x tailwindcss-linux-x64
}

function install_tailwind() {
    mv tailwindcss-linux-x64 /usr/local/sbin/tailwind
}

function run_tailwind() {
    tailwind -i style/tailwind.css -o style/output.css --watch
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
    tw|tailwind)
        run_tailwind
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|build|generate|download_tailwind|install_tailwind|tailwind}"
        exit 1
        ;;
esac

