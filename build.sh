if [[ $* ]]; then
  docker build . -t ghcr.io/thinkier/tcp-h264-server-armv7-build
fi

# Cross compile
docker run -v "$(pwd):/app" -v "$HOME/.cargo/registry:/root/.cargo/registry" ghcr.io/thinkier/tcp-h264-server-armv7-build
