FROM rust:1.93.0-slim-bullseye AS builder

COPY . /app

WORKDIR /app

RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder  /app/target/release/api /bin/app/api
WORKDIR /bin/app

CMD ["/bin/app/api"]
