# Channel-Bridge

[![CI](https://github.com/ivmarkov/channel-bridge/actions/workflows/ci.yml/badge.svg)](https://github.com/ivmarkov/channel-bridge/actions/workflows/ci.yml)
![crates.io](https://img.shields.io/crates/v/channel-bridge.svg)

Blocking and async `Sender` and `Receiver` traits. Async implementations:
* For [embassy-sync](https://github.com/embassy-rs/embassy/tree/main/embassy-sync)
* For the custom, lock-free `Notification` primitive offered in this crate
* For web sockets (based on WASM websockets, [embedded-svc](https://github.com/esp-rs/embedded-svc) or on [edge-net](https://github.com/ivmarkov/edge-net))
* For event bus (based on [embedded-svc](https://github.com/esp-rs/embedded-svc))
