# TODO

- [ ] Add `moonveil-core/src/session/mod.rs` implementing `Session`, `SessionState`, `SessionError`
- [ ] Update workspace `Cargo.toml` to add `uuid = { version = "1", features = ["v4"] }`
- [ ] Update `moonveil-core/Cargo.toml` to depend on `uuid` (via workspace)
- [ ] Update `moonveil-core/src/lib.rs` to export `session` module
- [ ] Update `moonveil-server/src/main.rs` to accept in a loop and spawn a task per connection using `Session`
- [ ] Update `moonveil-client/src/main.rs` to use `Session` instead of raw `Transport`, and log `session_id`
- [x] Run `cargo check` for the workspace (command unavailable in environment)


