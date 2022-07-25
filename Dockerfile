FROM rust:alpine as builder
WORKDIR /usr/src/myapp
RUN apk add --no-cache build-base
RUN apk add --no-cache pkgconfig openssl-dev

COPY Cargo.toml ./Cargo.toml
COPY Cargo.lock ./Cargo.lock
COPY .cargo ./.cargo
RUN mkdir ./src/
COPY main.rs.placeholder ./src/main.rs
RUN cargo install --path .

COPY src ./src
COPY sql ./sql
RUN cargo build --release

FROM alpine:edge
RUN apk update && apk add ca-certificates && rm -rf /var/lib/apt/lists/*
COPY .env /.env
COPY --from=builder /usr/src/myapp/target/release/discord-link /usr/local/bin/discord-link
EXPOSE 3000
CMD ["discord-link"]