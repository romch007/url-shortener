FROM rust:1-alpine AS builder

RUN apk add --no-cache musl-dev

RUN cargo install cargo-build-deps

ENV RUSTFLAGS='-C target-feature=-crt-static'

WORKDIR /app

RUN cargo new --bin url-shortener
WORKDIR /app/url-shortener

COPY Cargo.toml Cargo.lock ./
RUN cargo build-deps --release

COPY src ./src
RUN cargo build --release
RUN strip target/release/url-shortener

FROM alpine

ARG USER=default

RUN apk add --no-cache tini libgcc
RUN adduser -D $USER

ENV HOST=0.0.0.0
ENV PORT=8080

EXPOSE 8080

WORKDIR /app

USER $USER

COPY --from=builder /app/url-shortener/target/release/url-shortener ./

ENTRYPOINT [ "/sbin/tini", "--" ]

CMD ["/app/url-shortener"]
