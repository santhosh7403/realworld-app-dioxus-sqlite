
# Database Setup

Let's use the sqlx-cli command-line utility to help us easily drop, create, or reset the database specified by the DATABASE_URL in the .env file.

It is critical to set the DATABASE_URL environment variable before running any sqlx commands, as they operate directly on its value. This is typically done by running the source .env command from the project root folder.

To install sqlx-cli, run the command below. This assumes you already have the Rust toolchains installed; if not, please refer to the Rust toolchain section in the main README.

`cargo install sqlx-cli`  - this installs sqlx utility

Now, from the project root folder, run the following commands to create the database and run the initialization SQL scripts located in the migrations folder.

```
cd realworld-app-dioxus-sqlite

source .env

sqlx database setup
# The command above creates the DB and runs the migrations.
```

## Other Useful Commands

Here is a quick reference for other commands available with the sqlx utility:

```
santhosh@fedora:~/realworld-app-dioxus-sqlite$ sqlx 
Command-line utility for SQLx, the Rust SQL toolkit.

Usage: sqlx [OPTIONS] <COMMAND>

Commands:
  database     Group of commands for creating and dropping your database
  prepare      Generate query metadata to support offline compile-time verification
  migrate      Group of commands for creating and running migrations
  completions  Generate shell completions for the specified shell
  help         Print this message or the help of the given subcommand(s)

Options:
      --no-dotenv  Do not automatically load `.env` files
  -h, --help       Print help
  -V, --version    Print version
santhosh@fedora:~/realworld-app-dioxus-sqlite$ sqlx database
Group of commands for creating and dropping your database

Usage: sqlx database <COMMAND>

Commands:
  create  Creates the database specified in your DATABASE_URL
  drop    Drops the database specified in your DATABASE_URL
  reset   Drops the database specified in your DATABASE_URL, re-creates it, and runs any pending migrations
  setup   Creates the database specified in your DATABASE_URL and runs any pending migrations
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```
