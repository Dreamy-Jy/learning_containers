FROM rust:1.92.0-alpine3.23 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:3.23
COPY --from=builder /app/target/aarch64-unknown-linux-musl/release/scripts /usr/local/bin/scripts
CMD ["scripts"]
