FROM lieturd/backup-build:latest

ADD . /src

WORKDIR /src

RUN cargo test \
 && cargo build
