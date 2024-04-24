!/bin/bash

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

function test() {
    docker run --rm \
        --network magicdocs-net \
        -e HOST_URL=$HOST_URL \
        -e TEST_USER_USERNAME=$TEST_USER_USERNAME \
        -e TEST_USER_PASSWORD=$TEST_USER_PASSWORD \
        magicdocs_test_playwright:latest
}

function build() {
    docker build -t magicdocs_test_playwright:latest $DIR
}

function bt() {
    build
    test
    docker rmi magicdocs_test_playwright:latest
}

case $OP in
    test)
        test
        ;;
    build)
        build
        ;;
    bt)
        bt
        ;;
    *)
        echo "Usage: $0 {test|build}"
        exit 1
        ;;
esac