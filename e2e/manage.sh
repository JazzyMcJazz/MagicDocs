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

function test() {
    docker run --rm \
        --network magicdocs-net \
        -e HOST_URL=$HOST_URL \
        -e TEST_USER_USERNAME=$TEST_USER_USERNAME \
        -e TEST_USER_PASSWORD=$TEST_USER_PASSWORD \
        magicdocs_test_playwright:latest
}

function test_pipeline() {
    docker run -d --name playwright \
        --network magicdocs-net \
        -e HOST_URL=$HOST_URL \
        -e KEYCLOAK_TEST_USER_USERNAME=$KEYCLOAK_TEST_USER_USERNAME \
        -e KEYCLOAK_TEST_USER_PASSWORD=$KEYCLOAK_TEST_USER_PASSWORD \
        -e KEYCLOAK_TEST_ADMIN_USERNAME=$KEYCLOAK_TEST_ADMIN_USERNAME \
        -e KEYCLOAK_TEST_ADMIN_PASSWORD=$KEYCLOAK_TEST_ADMIN_PASSWORD \
        -e KEYCLOAK_TEST_SUPERADMIN_USERNAME=$KEYCLOAK_TEST_SUPERADMIN_USERNAME \
        -e KEYCLOAK_TEST_SUPERADMIN_PASSWORD=$KEYCLOAK_TEST_SUPERADMIN_PASSWORD \
        magicdocs_test_playwright:latest

    EXIT_CODE=$(docker wait playwright)
    echo "Playwright tests exited with code $EXIT_CODE"

    docker logs playwright
    docker cp playwright:/app/playwright-report/index.html $DIR/playwright-report/index.html
    docker cp playwright:/app/screenshots $DIR/screenshots
    docker rm playwright

    echo "EXIT_CODE: $EXIT_CODE"
    if [ $EXIT_CODE -ne 0 ]; then
        echo "Tests failed"
        exit 1
    fi

    echo "Tests passed"
}

function build() {
    docker build -t magicdocs_test_playwright:latest $DIR
}

function bt() {
    build
    # test
    test_pipeline
    docker rmi magicdocs_test_playwright:latest
}

case $OP in
    test)
        test
        ;;
    test_pipeline)
        test_pipeline
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