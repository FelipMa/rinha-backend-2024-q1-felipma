FROM rust:1.76.0-alpine3.19

RUN apk add musl-dev

WORKDIR /app

ADD ./Cargo.toml .
ADD ./Cargo.lock .
ADD ./src ./src

RUN cargo install --path .

CMD ["rinha-backend-2024-q1-felipma"]