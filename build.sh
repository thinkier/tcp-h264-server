#!/bin/sh
cargo install cross
cross build --release --target arm-unknown-linux-gnueabihf # For BCM2835 (Zero/W 1, A, A+ B, B+) and newer
#cross build --release --target armv7-unknown-linux-gnueabihf # For 3B+ and newer
#cross build --release --target aarch64-unknown-linux-gnu # For 4B and newer
