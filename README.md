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

### Web server

See [rest-api.http](rest-api.http) for examples of how to use the API.'

#### Run manually
```
$ screentimeapi serve
```

#### Run as a systemd service

Here is an example service file that you can adapt to your needs. It can be
placed in `~/.config/systemd/user/screentimeapi.service`:

```
[Unit]
Description=Screentime API
After=network.target

[Service]
Environment="DATABASE_URL=mysql://user:pass@localhost/screentimeapi"
Environment="SERVER_PORT=3000"
Environment="SERVER_ADDRESS=0.0.0.0"
ExecStart=/home/myuser/.local/bin/screentimeapi serve
Restart=always
RestartSec=5s
StartLimitBurst=5

[Install]
WantedBy=default.target
```

Once you've created this file, you can start the service with the following
command:

```
$ systemctl --user enable screentimeapi
```

To enable the service to start automatically at boot:

```
$ systemctl --user start screentimeapi
```
To check the status of your service:

```
$ systemctl --user status screentimeapi
```


Development
-----------

### Running tests

1. Create an empty test database with corresponding user.
1. Override the database URL with the test database while populating the tables
   and running tests:
    ```
    $ DATABASE_URL=mysql://screentimeapi:screentimeapi@localhost/screentimeapitest diesel migration run
    $ DATABASE_URL=mysql://screentimeapi:screentimeapi@localhost/screentimeapitest cargo test
    ```
