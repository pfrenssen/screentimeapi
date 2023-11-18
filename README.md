screentimeapi
=============


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
