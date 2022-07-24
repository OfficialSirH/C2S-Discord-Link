FROM rust:alpine as builder
WORKDIR /usr/src/myapp
COPY . .
RUN apk add build-base
RUN cargo install --path .
RUN cargo build --release

FROM debian:buster-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY .env /.env
COPY sql /sql
COPY --from=builder /usr/src/myapp/target/release/discordlink /usr/local/bin/myapp
EXPOSE 3000
CMD ["discordlink"]