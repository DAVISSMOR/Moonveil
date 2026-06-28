use crate::transport::{TransportError, TransportResult};

#[cfg(target_os = "linux")]
use tokio::io::AsyncReadExt;

#[cfg(target_os = "linux")]
pub struct TunDevice {
    name: String,
    mtu: u16,
    #[allow(dead_code)]
    fd: std::os::unix::io::RawFd,
}

#[cfg(not(target_os = "linux"))]
pub struct TunDevice {
    name: String,
    mtu: u16,
}

impl TunDevice {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn mtu(&self) -> u16 {
        self.mtu
    }
}

#[cfg(target_os = "linux")]
use std::os::unix::io::AsRawFd;

#[cfg(target_os = "linux")]
impl TunDevice {
    pub fn new(name: &str, mtu: u16) -> TransportResult<Self> {
        let dev = tun2::TunBuilder::new()
            .name(name)
            .mtu(mtu as i32)
            .try_build()
            .map_err(|e| {
                TransportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
            })?;

        let fd = dev.as_raw_fd();

        Ok(Self {
            name: name.to_string(),
            mtu,
            fd,
        })
    }

    pub async fn read_packet(&self) -> TransportResult<Vec<u8>> {
        use tokio::io::unix::AsyncFd;

        let mut file = AsyncFd::new(self.fd).map_err(|e| {
            TransportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        let mut buf = vec![0u8; 65535];
        let n = file
            .read(&mut buf)
            .await
            .map_err(|e| TransportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        buf.truncate(n);
        Ok(buf)
    }

    pub async fn write_packet(&self, data: &[u8]) -> TransportResult<()> {
        use tokio::io::unix::AsyncFd;

        let mut file = AsyncFd::new(self.fd).map_err(|e| {
            TransportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        let mut written = 0usize;
        while written < data.len() {
            let rc = file
                .write(&data[written..])
                .await
                .map_err(|e| {
                    TransportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                })?;
            written += rc;
        }
        Ok(())
    }

    pub fn set_ip_address(&self, cidr: &str) -> TransportResult<()> {
        let status1 = std::process::Command::new("ip")
            .args(["addr", "add", cidr, "dev", &self.name])
            .status()
            .map_err(|e| TransportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        if !status1.success() {
            return Err(TransportError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to set ip address via ip addr add {cidr} dev {}", self.name),
            )));
        }

        let status2 = std::process::Command::new("ip")
            .args(["link", "set", &self.name, "up"])
            .status()
            .map_err(|e| TransportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        if !status2.success() {
            return Err(TransportError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to bring up tun device {}", self.name),
            )));
        }

        Ok(())
    }

    pub fn enable_ip_forward() -> TransportResult<()> {
        use std::io::Write;

        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .open("/proc/sys/net/ipv4/ip_forward")
            .map_err(|e| TransportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        f.write_all(b"1")
            .map_err(|e| TransportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        Ok(())
    }

    pub fn setup_nat(subnet: &str) -> TransportResult<()> {
        let status = std::process::Command::new("iptables")
            .args([
                "-t",
                "nat",
                "-A",
                "POSTROUTING",
                "-s",
                subnet,
                "-j",
                "MASQUERADE",
            ])
            .status()
            .map_err(|e| TransportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        if !status.success() {
            return Err(TransportError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to setup NAT for subnet {subnet}"),
            )));
        }

        Ok(())
    }
}

#[cfg(not(target_os = "linux"))]
impl TunDevice {
    pub fn new(name: &str, mtu: u16) -> TransportResult<Self> {
        let _ = (name, mtu);
        Err(TransportError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "TUN interface is only supported on Linux",
        )))
    }

    pub async fn read_packet(&self) -> TransportResult<Vec<u8>> {
        Err(TransportError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "TUN interface is only supported on Linux",
        )))
    }

    pub async fn write_packet(&self, _data: &[u8]) -> TransportResult<()> {
        Err(TransportError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "TUN interface is only supported on Linux",
        )))
    }
}

pub mod forwarder;
pub use forwarder::IpForwarder;
