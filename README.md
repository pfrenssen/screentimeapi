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

To do POST requests, set the `Content-Type: application/json` header, and pass
the data in a JSON request body.

Routes:
- / - GET: returns the API version.
- /adjustment-types - GET: returns a list of all adjument types.
- /adjustment-types - POST: create an adjustment type.
