FROM rust:latest

RUN rustup target add wasm32-unknown-unknown

COPY ./src /faas/src/api/src
COPY ./Cargo.toml /faas/src/api/Cargo.toml
COPY ./Cargo.lock /faas/src/api/Cargo.lock
WORKDIR /faas/src/api
RUN cargo build --release
