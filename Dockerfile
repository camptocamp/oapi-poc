FROM rust:latest

RUN apt-get update && apt-get install -y clang cmake sqlite3 libsqlite3-dev

WORKDIR /app

COPY ./ .
