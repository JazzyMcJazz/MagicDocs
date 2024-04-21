#!/bin/bash
set -e

if [ -z "$KC_DB_USER" ] || [ -z "$KC_DB_PASS" ] || [ -z "$DB_DB_USER" ] || [ -z "$DB_DB_PASS" ]; then
    echo "Please set the environment variables KC_DB_USER, KC_DB_PASS, DB_DB_USER, and DB_DB_PASS."
    exit 1
fi

# Create the first database and user
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<-EOSQL
    CREATE DATABASE keycloak;
    CREATE USER $KC_DB_USER WITH ENCRYPTED PASSWORD '$KC_DB_PASS';
    GRANT ALL PRIVILEGES ON DATABASE keycloak TO $KC_DB_USER;
EOSQL

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" -d keycloak <<-EOSQL
    GRANT ALL ON SCHEMA public TO $KC_DB_USER;
EOSQL

# Create the second database and user
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<-EOSQL
    CREATE DATABASE magicdocs;
    CREATE USER $MD_DB_USER WITH ENCRYPTED PASSWORD '$MD_DB_PASS';
    GRANT ALL PRIVILEGES ON DATABASE magicdocs TO $MD_DB_USER;
EOSQL

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" -d magicdocs <<-EOSQL
    GRANT ALL ON SCHEMA public TO $MD_DB_USER;
EOSQL