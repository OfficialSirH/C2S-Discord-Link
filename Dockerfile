FROM rust:alpine as builder
WORKDIR /usr/src/myapp
COPY . .
RUN apk add build-base
RUN apk add pkgconfig openssl-dev
RUN cargo install --path .
RUN cargo build --release

FROM debian:buster-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY .env /.env
COPY sql /sql
COPY --from=builder /usr/src/myapp/target/release/discord-link /usr/local/bin/discord-link
EXPOSE 3000
CMD ["discord-link"]