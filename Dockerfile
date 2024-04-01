FROM rust:1.77-slim-bullseye AS builder
WORKDIR /usr/src/dimigomeal-api

COPY . .
RUN cargo build --release

FROM debian:bullseye

COPY --from=builder /usr/src/dimigomeal-api/target/release/dimigomeal-api /usr/local/bin/dimigomeal-api

CMD ["dimigomeal-api"]