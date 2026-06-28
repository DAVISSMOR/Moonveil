# Contributing

Thanks for taking the time to contribute to Moonveil Transport!

## Getting Started

1. **Clone the repository**
   
```bash
   git clone <REPO_URL>
   cd moonveil-transport
   ```

2. **Build**
   
```bash
   cargo build --workspace
   ```

3. **Run tests**
   
```bash
   cargo test --workspace
   ```

## Project Structure

- `moonveil-core/`: Core library (packet, session, mux, crypto, transports)
- `moonveil-client/`: Client CLI/binary
- `moonveil-server/`: Server CLI/binary

## How to add a new Transport

1. Add a new module under `moonveil-core/src/transport/` (e.g. `my_transport.rs`)
2. Implement the `Transport` trait in `moonveil-core/src/transport/mod.rs` exports.
3. Ensure your transport correctly implements:
   - `connect`
   - `send`
   - `recv`
   - `close`
4. Add any required configuration into `moonveil-core/src/config.rs` / config files.

## How to add a new Cipher

1. Add a new module under `moonveil-core/src/crypto/` (e.g. `my_cipher.rs`)
2. Implement the `Cipher` trait:
   - `encrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError>`
   - `decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError>`
3. Export the cipher from `moonveil-core/src/crypto/mod.rs`.

## Code style

- Run formatting:
  
```bash
  cargo fmt
  
```
- Run linting:
  
```bash
  cargo clippy --workspace --all-targets -- -D warnings
  ```

## Pull Request process

1. Create a feature branch:
   
```bash
   git checkout -b blackboxai/<your-branch-name>
   
```
2. Commit your changes with clear messages.
3. Open a PR with:
   - Description of the change
   - Why it’s needed
   - How it was tested (`cargo test --workspace`, and `cargo bench` if applicable)
4. Address review comments and re-run checks before merging.
