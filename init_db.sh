#!/bin/bash
set -e

DB_NAME="hs_blog"
DB_USER="root"
DB_PASSWORD="rootroot"

# Check whether the database already exists
DB_EXISTS=$(sudo -u postgres psql -tAc "SELECT 1 FROM pg_database WHERE datname='${DB_NAME}'")

if [ "$DB_EXISTS" = "1" ]; then
	echo "Database '$DB_NAME' already exists. Skipping database creation."
else
	sudo -u postgres psql -c "CREATE DATABASE $DB_NAME;"
	echo "Database '$DB_NAME' created."
fi

# Check whether the user already exists
USER_EXISTS=$(sudo -u postgres psql -tAc "SELECT 1 FROM pg_roles WHERE rolname='${DB_USER}'")

if [ "$USER_EXISTS" = "1" ]; then
	echo "User '$DB_USER' already exists. Skipping user creation."
else
	sudo -u postgres psql -c "CREATE USER $DB_USER WITH ENCRYPTED PASSWORD '$DB_PASSWORD';"
	echo "User '$DB_USER' created."
fi

# Grant privileges to the user on the database
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;"

echo "Privileges granted on database '$DB_NAME' to user '$DB_USER'."

