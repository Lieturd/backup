# Lieturd backup

# backup-cli [![Build Status](https://travis-ci.org/Lieturd/backup-cli.svg?branch=master)](https://travis-ci.org/Lieturd/backup-cli)
# backupd [![Build Status](https://travis-ci.org/Lieturd/backupd.svg?branch=master)](https://travis-ci.org/Lieturd/backupd)
# backuplib [![Build Status](https://travis-ci.org/Lieturd/backup-cli.svg?branch=master)](https://travis-ci.org/Lieturd/backup-cli)

## Building the Docker build environment base image

```bash
cd backup-build
docker build -t lieturd/backup-build:YYYY-MM-DD -t lieturd/backup-build:latest .
docker push lieturd/backup-build:YYYY-MM-DD lieturd/backup-build:latest
```
