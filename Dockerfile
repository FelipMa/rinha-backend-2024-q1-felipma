FROM rust:1.76.0-alpine3.19

RUN apk add musl-dev

WORKDIR /app

ADD ./Cargo.toml .
ADD ./Cargo.lock .
ADD ./src ./src

RUN cargo build --release

CMD ["./target/release/rinha-backend-2024-q1-felipma"]