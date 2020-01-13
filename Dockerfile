FROM rust:1.39 as builder

RUN rustup target add x86_64-unknown-linux-musl \ 
    && apt update \ 
    && apt upgrade -y \
    && apt-get install -y musl musl-dev musl-tools librust-openssl-dev librust-openssl-sys-dev libssl-dev upx-ucl

ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src/build
COPY Cargo.* ./
RUN mkdir .cargo \
    && cargo vendor > .cargo/config

COPY . .

RUN cargo build --release \
	&& cargo build --release --target=x86_64-unknown-linux-musl \
	&& strip ./target/release/ssm_helper \
	&& upx -9 ./target/release/ssm_helper \
	&& strip ./target/x86_64-unknown-linux-musl/release/ssm_helper \
	&& upx -9 ./target/x86_64-unknown-linux-musl/release/ssm_helper
	
FROM alpine:3.10

COPY --from=builder /usr/src/build/target/x86_64-unknown-linux-musl/release/ssm_helper /usr/bin/