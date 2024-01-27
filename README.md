# Channel-Bridge

[![CI](https://github.com/ivmarkov/channel-bridge/actions/workflows/ci.yml/badge.svg)](https://github.com/ivmarkov/channel-bridge/actions/workflows/ci.yml)
![crates.io](https://img.shields.io/crates/v/channel-bridge.svg)

Blocking and async `Sender` and `Receiver` traits with particular emphasis on async. 

Async implementations:
* For a lot of the synchronization primitives in [embassy-sync](https://github.com/embassy-rs/embassy/tree/main/embassy-sync)
* For the custom, lock-free [`Notification`](src/notification.rs) primitive offered in this crate
* For web sockets
  * For WASM websockets
  * For [edge-ws](https://github.com/ivmarkov/edge-net/tree/master/edge-ws)
  * For anything else that happens to implement the [`embedded-svc` Websocket traits](https://github.com/esp-rs/embedded-svc/blob/master/src/ws.rs#L114)
