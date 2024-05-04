#!/bin/bash
set -e

if [ -z "$MD_DB_USER" ] || [ -z "$MD_DB_PASS" ]; then
    echo "Please set the environment variables DB_DB_USER, and DB_DB_PASS."
    exit 1
fi

# Create the second database and user
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<-EOSQL
    CREATE DATABASE magicdocs;
    CREATE USER $MD_DB_USER WITH ENCRYPTED PASSWORD '$MD_DB_PASS';
    GRANT ALL PRIVILEGES ON DATABASE magicdocs TO $MD_DB_USER;
EOSQL

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" -d magicdocs <<-EOSQL
    GRANT ALL ON SCHEMA public TO $MD_DB_USER;
EOSQL