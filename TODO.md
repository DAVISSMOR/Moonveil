# Fix Plan — 3 Critical Issues ✅ ALL DONE

## Issue 1: TunDevice is not Clone ✅
- [x] **moonveil-core/src/tun/forwarder.rs** — `new()` now accepts `Arc<TunDevice>` directly, removed `with_shared_tun()`
- [x] **moonveil-server/src/main.rs** — Switched from `with_shared_tun()` to `new()`
- [x] **moonveil-client/src/main.rs** — Added `use std::sync::Arc`, wrapped `TunDevice` in `Arc`, uses `Arc::clone()`

## Issue 2: QUIC undefined behavior ✅
- [x] **moonveil-core/src/transport/quic.rs** — Removed `unsafe` `MaybeUninit`, removed `Endpoint` field, removed `quinn` import, added doc comment

## Issue 3: frame.rs unwrap() on untrusted data ✅
- [x] **moonveil-core/src/frame.rs** — All 5 `.try_into().unwrap()` replaced with `map_err` + `FrameError::FrameDecodeError`; test block unwraps left untouched

## Verification ⚠️
- [ ] `cargo check --workspace` — Cannot run (Rust not installed on this Windows system)
- [ ] `cargo test --workspace` — Cannot run (Rust not installed on this Windows system)
