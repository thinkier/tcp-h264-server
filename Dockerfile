# ARMv7 cross compiler docker image

FROM rust:latest

RUN apt update && apt upgrade -y
RUN apt install -y g++-arm-linux-gnueabihf libc6-dev-armhf-cross

RUN rustup target add armv7-unknown-linux-gnueabihf
RUN rustup toolchain install nightly-armv7-unknown-linux-gnueabihf
RUN rustup default nightly
RUN rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu

WORKDIR /app

ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc \
    CC_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc \
    CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++

CMD cargo build --release --target armv7-unknown-linux-gnueabihf -Zbuild-std