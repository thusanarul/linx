FROM rust:1.93.0-slim-bullseye AS builder

COPY . /app

WORKDIR /app

RUN cargo build --release

FROM debian:bullseye-slim

COPY --from=builder  /app/target/release/api /bin/app/api
WORKDIR /bin/app

CMD ["/bin/app/api"]
