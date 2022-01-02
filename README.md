# Multiplexable/Reusable TCP H.264 Server

## Problem

`raspivid` and `libcamera-vid` only emits its outputs to 1 TCP stream and exits when that session terminates. This server aims to capture that H.264 stream and serve it to multiple clients, allow reconnection, and other actions that aren't possible with `raspivid` and `libcamera-vid` alone.

## Goals

- [x] Allow multiple concurrent connection
- [x] Start stream correctly at a convenient position

## Stretch Goals

- [ ] Snapshot provider (for integration with `homebridge-camera-ffmpeg`), which hooks into `raspicam` or `libcamera-png` when the stream isn't active, or takes a pic from the video stream.

## Notes

- Requires the emitter of the H264 stream to have redundant slices (NAL type 0x5) (ie. it must be either `baseline` or `extended` profile)
