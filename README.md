# Multiplexable/Reusable TCP H.264 Server

## Problem

`raspivid` and `libcamera-vid` only emits its outputs to 1 TCP stream and exits when that session terminates. This
server aims to capture that H.264 stream and serve it to multiple clients, allow reconnection, and other actions that
aren't possible with `raspivid` and `libcamera-vid` alone.

## Goals

- [x] Allow multiple concurrent connection
- [x] Start stream correctly at a convenient position

## Stretch Goals

- [x] Snapshot provider (for integration with `homebridge-camera-ffmpeg`)
    - [ ] Hook into `raspicam` or `libcamera-png` when the stream isn't active
- [ ] Host the `raspicam` / `libcamera-vid` process
  - [ ] Automatically restart the process (my CSI cable is 2m+ long and drops out sporadically :/)
- [ ] Adjust exposure automatically

## Sample `homebridge-camera-ffmpeg` configuration

```json
{
  "name": "Front Porch",
  "videoConfig": {
    "source": "-i tcp://192.168.1.169:1264",
    "stillImageSource": "-i http://192.168.1.169:8080/"
  }
}
```

## Notes

- Requires the emitter of the H264 stream to have redundant slices (NAL type 0x5) (ie. it must be either `baseline`
  or `extended` profile)
