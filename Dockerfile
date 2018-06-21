FROM rust:1.28.0 as builder

RUN rustup target add x86_64-unknown-linux-musl 

WORKDIR /usr/src/build
COPY . .

RUN cargo build --release \
	&& cargo build --release --target=x86_64-unknown-linux-musl