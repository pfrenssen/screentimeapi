screentimeapi
=============

This is a personal project that provides an API to keep track of how much
screen time a child has earned by doing their chores. The available screen time
can be adjusted upwards or downwards depending on whether they did their chores.

Requirements
------------

* MySQL or MariaDB
* Cargo
* Diesel CLI (`cargo install diesel_cli --no-default-features --features mysql`)


Installation
------------

1. Create an empty database and a database user that has all privileges on the
   database.
1. Copy `.env.dist` to `.env` and adapt it to include your database name and
   user credentials.
1. Create database tables:
    ```
    $ diesel migration run
    ```
1. Compile:
    ```
    $ cargo build --release
    ```
1. Symlink the binary somewhere in your `$PATH`, for example:
    ```
    $ ln -rs target/release/screentimeapi ~/.local/bin/
    ```


Usage
-----

### CLI

```
$ screentimeapi --help
```

### REST API

Start the server.
```
$ screentimeapi serve
```

See [rest-api.http](rest-api.http) for examples.


### Running tests

1. Create an empty test database with corresponding user.
1. Override the database URL with the test database while populating the tables
   and running tests:
    ```
    $ DATABASE_URL=mysql://screentimeapi:screentimeapi@localhost/screentimeapitest diesel migration run
    $ DATABASE_URL=mysql://screentimeapi:screentimeapi@localhost/screentimeapitest cargo test
    ```
