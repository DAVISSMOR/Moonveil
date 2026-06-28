<div align="center">

```
        ◐
   Moonveil
```

**A modular transport platform built for the modern internet.**

[![Rust](https://img.shields.io/badge/Rust-1.96-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue?style=flat-square)](LICENSE)
[![Tests](https://img.shields.io/badge/Tests-20%20passing-brightgreen?style=flat-square)](#testing)
[![Version](https://img.shields.io/badge/Version-0.1.0-purple?style=flat-square)](#roadmap)

*Adaptive by design. Built for freedom.*

</div>

---

## What is Moonveil?

Moonveil is not a VPN. It is not a replacement for QUIC.

It is a **transport platform** — a modular foundation that lets you swap protocols, encryption layers, and routing strategies without touching the core. Think of it as a constructor kit for modern network communication.

The guiding idea: in 5–10 years, you should be able to replace any module without rewriting the core.

```
              Moonveil
                  │
      ┌───────────┴───────────┐
      │                       │
   Moonveil CLI         Moonveil GUI
      │                       │
      └───────────┬───────────┘
                  │
            Moonveil Core
                  │
   ┌──────────────┼──────────────┐
   │              │              │
Session       Scheduler      Multiplexer
   │              │              │
   └──────────────┼──────────────┘
                  │
               Crypto
                  │
        Transport Abstraction
   ┌────────┬────────┬────────┐
   │        │        │        │
  UDP      TCP     HTTP/3   QUIC
```

The Core knows nothing about the transport layer. It speaks one language:

```rust
trait Transport {
    async fn send(&self, packet: Packet) -> Result<(), TransportError>;
    async fn recv(&self) -> Result<Packet, TransportError>;
    async fn connect(&self) -> Result<(), TransportError>;
    async fn close(&self) -> Result<(), TransportError>;
}
```

That's it. You could write a transport over a serial port or a satellite link — Core stays the same.

---

## Architecture

```
moonveil/
├── moonveil-core/
│   ├── src/
│   │   ├── transport/       # Transport trait + TCP/UDP/QUIC implementations
│   │   ├── session/         # Session lifecycle management
│   │   ├── mux/             # Multiplexer — multiple sessions in parallel
│   │   ├── crypto/          # AES-256-GCM encryption layer
│   │   ├── config.rs        # TOML configuration
│   │   ├── packet.rs        # Core packet structure
│   │   └── frame.rs         # Binary framing with CRC32
│   └── tests/
│       └── integration_test.rs
├── moonveil-client/         # Client binary with CLI
├── moonveil-server/         # Server binary with CLI
└── config/
    ├── client.toml
    └── server.toml
```

### Key design decisions

**Layered crypto** — `EncryptedTransport` wraps any transport transparently. Session has no idea encryption exists.

**Plugin-ready** — Every major subsystem (scheduler, crypto, transport, compression) is designed to be swapped via traits. Researchers can change individual algorithms without touching the rest.

**Zero unsafe** — Built entirely in safe Rust with tokio async runtime.

---

## Getting Started

### Prerequisites

- Rust 1.75+ (`rustup install stable`)
- For Windows: MinGW toolchain (`rustup default stable-x86_64-pc-windows-gnu`)

### Build

```bash
git clone https://github.com/DAVISSMOR/Moonveil.git
cd Moonveil
cargo build --release
```

### Run the server

```bash
cargo run --bin moonveil-server -- start --config config/server.toml
```

### Run the client

```bash
cargo run --bin moonveil-client -- connect --config config/client.toml
```

### Configuration

Edit `config/server.toml` or `config/client.toml`:

```toml
[server]
host = "127.0.0.1"
port = 7878

[crypto]
preshared_key = "your-64-char-hex-key-here"

[transport]
mode = "tcp"   # tcp | udp | quic (coming soon)

[log]
level = "info" # debug | info | warn | error
```

---

## Testing

```bash
cargo test --workspace
```

```
running 19 tests
test crypto::cipher::tests::encrypt_decrypt_roundtrip ... ok
test crypto::cipher::tests::decrypt_with_wrong_key_fails ... ok
test session::tests::session_close_transitions_to_closed ... ok
test mux::tests::add_session_increases_count ... ok
test mux::tests::remove_session_decreases_count ... ok
... and 14 more

test result: ok. 19 passed; 0 failed

running 1 test
test test_client_server_roundtrip ... ok

test result: ok. 1 passed; 0 failed
```

---

## Roadmap

| Version | Status | Description |
|---------|--------|-------------|
| **v0.1** | ✅ Done | Stable core — Transport, Session, Mux, Crypto, CLI |
| **v0.2** | 🔧 In progress | Performance benchmarks, full documentation, metrics |
| **v0.5** | 📋 Planned | UDP transport, traffic obfuscation, plugin system, public alpha |
| **v1.0** | 📋 Planned | QUIC transport, GUI, CI/CD, cross-platform binaries |

---

## Why Moonveil?

The internet is increasingly fragmented. Traffic inspection, deep packet analysis, and protocol-level blocking are becoming standard tools of control.

Moonveil is designed from the ground up to be **adaptive** — when one transport is blocked, you switch to another. When a cipher is compromised, you replace it. The platform survives because no single layer is irreplaceable.

This is not just a technical project. It is infrastructure for **a free and open internet**.

> *"The Net interprets censorship as damage and routes around it."*
> — John Gilmore, 1993

---

## Contributing

Moonveil is at an early stage and welcomes contributions. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Areas where help is most needed:
- UDP transport implementation
- Traffic obfuscation layer
- Cross-platform testing
- Documentation and examples

---

## License

MIT License — see [LICENSE](LICENSE) for details.

---

<div align="center">

**Moonveil** — *Performance through simplicity.*

Built with Rust 🦀

</div>
