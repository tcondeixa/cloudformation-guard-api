FROM rust:1.65.0-slim AS builder
WORKDIR /usr/src/cloudformation-guard-api
COPY Cargo.toml Cargo.lock ./
COPY src src/
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:3.16 AS runtime
COPY --from=builder /usr/src/cloudformation-guard-api/target/x86_64-unknown-linux-musl/release/cloudformation-guard-api /usr/local/bin
EXPOSE 8080
CMD [ "/usr/local/bin/cloudformation-guard-api"]
