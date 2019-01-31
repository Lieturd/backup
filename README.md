# Lieturd backup [![Build Status](https://travis-ci.org/Lieturd/backup.svg?branch=master)](https://travis-ci.org/Lieturd/backup)

## Building the Docker build environment base image

```bash
cd backup-build
docker build -t lieturd/backup-build:YYYY-MM-DD -t lieturd/backup-build:latest .
docker push lieturd/backup-build:YYYY-MM-DD lieturd/backup-build:latest
```

## Building and running server and client

```bash
# To run the server:
cargo run --release --bin backupd [BACKUP_DIRECTORY]

# To run the client:
cargo run --release --bin backup-cli [FILE_PATH_TO_UPLOAD]
```

Since the client uses path of the file to upload to tell the server where to
put the file, it's best if the resides within the current working directory and
you use a relative path to the file.
