# Lieturd backup [![Build Status](https://travis-ci.org/Lieturd/backup.svg?branch=master)](https://travis-ci.org/Lieturd/backup)

## Building the Docker build environment base image

```bash
cd backup-build
docker build -t lieturd/backup-build:YYYY-MM-DD -t lieturd/backup-build:latest .
docker push lieturd/backup-build:YYYY-MM-DD lieturd/backup-build:latest
```

## Building and running server and client

The server requires an SQLite database to work. It must include a table
formatted as such:

```sql
CREATE TABLE files (
    id TEXT NOT NULL PRIMARY KEY,
    filename TEXT NOT NULL,
    last_modified BIGINT NOT NULL
);
```

First start up the server, then run the client. This will cause the client to
check the backup paths once a minute to upload updated files.

```bash
# To run the server:
cargo run --release --bin backupd [DATABASE_DIRECTORY]

# To run the client:
cargo run --release --bin backup-cli [FILE_PATH_TO_UPLOAD]
```

# Development

Development is easier without the `--release` mode, so just run in
separate terminals or similar:

```bash
cargo run --bin backupd
cargo run --bin backup-cli
```
