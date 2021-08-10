# Bale Proxy

[![Crates.io](https://img.shields.io/crates/v/bale-proxy)](https://crates.io/crates/bale-proxy)
[![Documentation](https://docs.rs/bale-proxy/badge.svg)](https://docs.rs/bale-proxy)
[![License](https://img.shields.io/crates/l/bale-proxy)](LICENSE)

A secure channel on [Bale](https://bale.ai/) messenger to use as a plugin for [shadowsocks](https://crates.io/crates/shadowsocks-rust).

## Why?

Because of censorship that may happen one day in Iran
that blocks every type of traffic to outside world except
some government-backed messengers.

## Testing

To run the tests, first start the server:

```bash
RUST_LOG=info cargo run -p test-server
```

Then, after the server is built and running, the tests can be run.

```bash
wasm-pack test --firefox --chrome --safari --headless test/test-client
```
