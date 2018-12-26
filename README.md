# Lieturd backup [![Build Status](https://travis-ci.org/Lieturd/backup.svg?branch=master)](https://travis-ci.org/Lieturd/backup)

## Building the Docker build environment base image

```bash
cd backup-build
docker build -t lieturd/backup-build:YYYY-MM-DD -t lieturd/backup-build:latest .
docker push lieturd/backup-build:YYYY-MM-DD lieturd/backup-build:latest
```
