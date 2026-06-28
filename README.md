<div align="center">

```
        ◐
   Moonveil
```

**A modular transport platform built for the modern internet.**

[![Rust](https://img.shields.io/badge/Rust-1.96-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue?style=flat-square)](LICENSE)
[![Tests](https://img.shields.io/badge/Tests-20%20passing-brightgreen?style=flat-square)](#testing)
[![Version](https://img.shields.io/badge/Version-1.1.0-purple?style=flat-square)](#roadmap)
[![CI](https://img.shields.io/github/actions/workflow/status/DAVISSMOR/Moonveil/ci.yml?style=flat-square&label=CI)](https://github.com/DAVISSMOR/Moonveil/actions)

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
      │                       │  (coming soon)
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
          Obfuscation Layer
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

---

## Architecture

```
moonveil/
├── moonveil-core/
│   ├── src/
│   │   ├── transport/       # Transport trait + TCP/UDP/QUIC
│   │   ├── session/         # Session lifecycle + metrics
│   │   ├── mux/             # Multiplexer — multiple sessions
│   │   ├── crypto/          # AES-256-GCM encryption layer
│   │   ├── obfuscation/     # Padding + XOR obfuscation
│   │   ├── tun/             # TUN interface + IP forwarder
│   │   ├── config.rs        # TOML configuration
│   │   ├── packet.rs        # Core packet structure
│   │   └── frame.rs         # Binary framing with CRC32
│   ├── benches/             # Criterion benchmarks
│   └── tests/
│       └── integration_test.rs
├── moonveil-client/         # Client binary with CLI
├── moonveil-server/         # Server binary with CLI
├── scripts/
│   └── install-server.sh    # Ubuntu 22.04 install script
└── config/
    ├── client.toml
    ├── server.toml
    └── server-tun.toml      # TUN/VPN mode config
```

---

## Getting Started

### Prerequisites

- Rust 1.75+ (`rustup install stable`)
- For Windows: MinGW toolchain (`rustup default stable-x86_64-pc-windows-gnu`)
- For TUN/VPN mode: Linux (Ubuntu 22.04+) with root access

### Build

```bash
git clone https://github.com/DAVISSMOR/Moonveil.git
cd Moonveil
cargo build --release
```

### Quick install on Ubuntu 22.04 (server)

```bash
git clone https://github.com/DAVISSMOR/Moonveil.git
cd Moonveil
chmod +x scripts/install-server.sh
sudo ./scripts/install-server.sh
```

This will:
- Build the release binary
- Install to `/usr/local/bin/moonveil-server`
- Enable IP forwarding
- Setup iptables NAT rules
- Create and start a systemd service

---

## VPN Mode (TUN Tunneling)

Moonveil can route **all system traffic** through an encrypted tunnel — no SOCKS proxy required, no app configuration needed.

### Server setup (Ubuntu 22.04)

```bash
sudo moonveil-server tun \
  --config config/server-tun.toml \
  --tun-name moonveil0 \
  --tun-addr 10.8.0.1/24
```

This automatically:
- Creates TUN interface `moonveil0`
- Enables IP forwarding (`/proc/sys/net/ipv4/ip_forward`)
- Sets up NAT masquerade via iptables
- Listens for encrypted client connections

### Client setup (Linux)

```bash
sudo moonveil-client tun \
  --config config/client.toml \
  --tun-name moonveil0 \
  --tun-addr 10.8.0.2/24 \
  --server YOUR_SERVER_IP:7878
```

This automatically:
- Creates TUN interface `moonveil0`
- Routes all traffic through the tunnel
- Encrypts everything with AES-256-GCM

### Traffic flow

```
Your apps (browser, telegram, games...)
              ↓
    TUN interface (moonveil0)
              ↓
    IpForwarder reads IP packets
              ↓
    AES-256-GCM encryption
              ↓
    Obfuscation layer
              ↓
    TCP/UDP/QUIC transport
              ↓
    Your Moonveil server
              ↓
         Internet
```

---

## Proxy Mode (TCP)

For quick testing without TUN:

### Run the server

```bash
cargo run --bin moonveil-server -- start --config config/server.toml
```

### Run the client

```bash
cargo run --bin moonveil-client -- connect --config config/client.toml
```

---

## Configuration

Edit `config/server-tun.toml` for VPN mode:

```toml
[server]
host = "0.0.0.0"
port = 7878

[crypto]
preshared_key = "your-64-char-hex-key-here"

[transport]
mode = "tcp"   # tcp | udp | quic

[tun]
name = "moonveil0"
address = "10.8.0.1/24"
mtu = 1500

[log]
level = "info"
```

---

## Obfuscation

Moonveil hides traffic patterns from Deep Packet Inspection (DPI):

| Technique | Description | Status |
|-----------|-------------|--------|
| Packet Padding | Random bytes hide real packet size | ✅ v0.5.0 |
| XOR Obfuscation | Hides data patterns | ✅ v0.5.0 |
| Traffic Mimicry | Traffic looks like HTTPS/TLS | 📋 v1.2 |
| MycelVeil Engine | Adaptive multi-path routing | 📋 v2.0 |

### MycelVeil — coming soon

MycelVeil is Moonveil's next-generation adaptive transport engine, inspired by how mycelium (fungal networks) finds the most efficient path through obstacles:

```
MycelVeil Engine
├── 🌱 Spore     — probes all paths, finds what's alive
├── 🍄 Mycel     — builds a live map, reroutes instantly
├── 🧬 Skin      — adapts traffic appearance per path
│   ├── HTTP Skin    → looks like a browser
│   ├── gRPC Skin    → looks like corporate API
│   └── Stream Skin  → looks like video streaming
└── 🌿 Branch    — multiple parallel paths, zero downtime
```

If one path gets blocked → MycelVeil switches instantly. The user notices nothing.

---

## Testing

```bash
cargo test --workspace
```

```
running 20 tests
test crypto::cipher::tests::encrypt_decrypt_roundtrip ... ok
test crypto::cipher::tests::decrypt_with_wrong_key_fails ... ok
test session::tests::session_close_transitions_to_closed ... ok
test mux::tests::add_session_increases_count ... ok
test transport::udp::tests::udp_send_recv_roundtrip ... ok
test obfuscation::tests::padding_roundtrip ... ok
... and 14 more

test result: ok. 20 passed; 0 failed

running 1 test
test test_client_server_roundtrip ... ok

test result: ok. 1 passed; 0 failed
```

---

## Roadmap

| Version | Status | Description |
|---------|--------|-------------|
| **v0.1** | ✅ Done | Stable core — Transport, Session, Mux, Crypto, CLI |
| **v0.2** | ✅ Done | Benchmarks, session metrics, documentation |
| **v0.5** | ✅ Done | UDP transport, obfuscation layer, CI/CD |
| **v1.0** | ✅ Done | QUIC transport, cross-platform binaries, CHANGELOG |
| **v1.1** | ✅ Done | TUN/VPN tunneling, IpForwarder, install script |
| **v1.2** | 🔧 Next | ACK/retransmit, DNS protection, gRPC transport |
| **v1.3** | 📋 Planned | SOCKS5 proxy, Split Tunneling, GeoIP routing |
| **v2.0** | 📋 Planned | MycelVeil engine, GUI, Web Panel |
| **v2.5** | 📋 Planned | Relay nodes, SDK (Go/Python), mobile apps |
| **v3.0** | 📋 Planned | Plugin marketplace, cloud ecosystem |

---

## Why Moonveil?

The internet is increasingly fragmented. Traffic inspection, deep packet analysis, and protocol-level blocking are becoming standard tools of control.

Moonveil is designed from the ground up to be **adaptive** — when one transport is blocked, you switch to another. When a cipher is compromised, you replace it. When DPI learns a new pattern, MycelVeil adapts.

The platform survives because no single layer is irreplaceable.

> *"The Net interprets censorship as damage and routes around it."*
> — John Gilmore, 1993

---

## Contributing

Moonveil welcomes contributions. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Areas where help is most needed:
- gRPC transport implementation
- MycelVeil engine (Spore/Mycel/Skin/Branch)
- Split Tunneling + GeoIP routing
- Cross-platform TUN support
- Documentation and examples

---

## License

MIT License — see [LICENSE](LICENSE) for details.

---

<div align="center">

**Moonveil** — *Adaptive by design. Built for freedom.*

Built with Rust 🦀 🍄

</div>
