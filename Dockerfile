FROM rust:1 AS builder

RUN cargo install cargo-build-deps

WORKDIR /app

RUN cargo new --bin url-shortener
WORKDIR /app/url-shortener

COPY Cargo.toml Cargo.lock ./
RUN cargo build-deps --release

COPY src ./src
RUN cargo build --release
RUN strip target/release/url-shortener

ENV TINI_VERSION=v0.19.0
ADD https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini-static /tini
RUN chmod +x /tini

FROM gcr.io/distroless/cc-debian12:nonroot

ENV HOST=0.0.0.0
ENV PORT=8080

EXPOSE 8080

WORKDIR /app

COPY --from=builder /app/url-shortener/target/release/url-shortener /app
COPY --from=builder /tini /tini

ENTRYPOINT ["/tini" , "--"]

CMD ["/app/url-shortener"]
