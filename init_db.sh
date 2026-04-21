#!/bin/bash
set -e

DB_NAME="hs_blog"
DB_USER="hs_user"
DB_PASSWORD="0d000721"

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
	# Create the user with the specified password
	sudo -u postgres psql -c "CREATE USER $DB_USER WITH ENCRYPTED PASSWORD '$DB_PASSWORD';"
	echo "User '$DB_USER' created."
fi

# Grant privileges to the user on the database
sudo -u postgres psql -U postgres -d $DB_NAME -c "GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;"  # Grant all privileges on the database
sudo -u postgres psql -U postgres -d $DB_NAME -c "GRANT CREATE ON SCHEMA public TO $DB_USER;"  # Grant CREATE privilege on the public schema to allow creating tables
sudo -u postgres psql -U postgres -d $DB_NAME -c "GRANT USAGE ON SCHEMA public TO $DB_USER;"  # Grant USAGE privilege on the public schema to allow using it
sudo -u postgres psql -U postgres -d $DB_NAME -c "GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO $DB_USER;"  # Grant all privileges on existing tables in the public schema

echo "Privileges granted on database '$DB_NAME' to user '$DB_USER'."

