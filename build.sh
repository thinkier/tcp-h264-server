#!/bin/sh
# Cross compile
docker run -v "$(pwd):/app" ghcr.io/thinkier/tcp-h264-server-armv7-build
