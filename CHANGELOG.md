# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.0] - 2025-01-16
* Breaking: 
  * Update `edge-ws` to 0.4

## [0.8.0] - 2024-02-01
* Breaking changes in `asynch::ws` module:
  * Replace the `const N: usize` parameter of all senders and receivers with a `&'a mut [u8]` buffer provided externally. Reason: this provides an option to (statically) pre-allocate the buffers outside of the async code thus resulting in futures' size reduction.
  * Re-implement the `accept` function as `Acceptor::run`, where `Acceptor` is a struct having these buffers managed internally
* Breaking change: feature `edge-net` renamed to `edge-ws`
* Depend on `edge-ws` instead of on all of the `edge-net`
