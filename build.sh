#!/bin/sh

if [[ $* ]]; then
  docker build . -t ghcr.io/thinkier/tcp-h264-server-armv7-build
fi

# Cross compile
docker run -v "$(pwd):/app" ghcr.io/thinkier/tcp-h264-server-armv7-build
