#!/bin/bash
set -e
echo "Installing Moonveil Server..."

# Install Rust if not present
if ! command -v cargo &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# Build release binary
cargo build --release -p moonveil-server

# Install binary
cp target/release/moonveil-server /usr/local/bin/
chmod +x /usr/local/bin/moonveil-server

# Enable IP forwarding permanently
echo "net.ipv4.ip_forward=1" >> /etc/sysctl.conf
sysctl -p

# Create config directory
mkdir -p /etc/moonveil
cp config/server-tun.toml /etc/moonveil/server.toml

# Create systemd service
cat > /etc/systemd/system/moonveil.service << EOF
[Unit]
Description=Moonveil VPN Server
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/moonveil-server tun --config /etc/moonveil/server.toml --tun-name moonveil0 --tun-addr 10.8.0.1/24
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable moonveil
systemctl start moonveil

echo "Moonveil Server installed successfully!"
echo "Server running on port 7878"
