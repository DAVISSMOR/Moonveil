use std::os::fd::FromRawFd;
use crate::transport::{TransportError, TransportResult};

#[cfg(target_os = "linux")]
pub struct TunDevice {
    name: String,
    mtu: u16,
    fd: std::os::fd::OwnedFd,
}

#[cfg(not(target_os = "linux"))]
pub struct TunDevice {
    name: String,
    mtu: u16,
}

impl TunDevice {
    pub fn name(&self) -> &str { &self.name }
    pub fn mtu(&self) -> u16 { self.mtu }
}

#[cfg(target_os = "linux")]
impl TunDevice {
    pub fn new(name: &str, mtu: u16) -> TransportResult<Self> {
        let mut config = tun2::Configuration::default();
        config.tun_name(name).mtu(mtu as u16).up();

        let dev = tun2::create(&config).map_err(|e| {
            TransportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        use std::os::fd::IntoRawFd;
        let raw_fd = dev.into_raw_fd();
        let owned = unsafe { std::os::fd::OwnedFd::from_raw_fd(raw_fd) };

        Ok(Self { name: name.to_string(), mtu, fd: owned })
    }

    pub async fn read_packet(&self) -> TransportResult<Vec<u8>> {
        use std::os::fd::AsRawFd;
        use tokio::io::unix::AsyncFd;
        use std::io::Read;

        let raw = self.fd.as_raw_fd();
        let afd = AsyncFd::new(raw).map_err(|e| TransportError::Io(e))?;

        loop {
            let mut guard = afd.readable().await.map_err(|e| TransportError::Io(e))?;
            let mut buf = vec![0u8; 65535];
            match guard.try_io(|_| {
                let mut f = unsafe { std::fs::File::from_raw_fd(raw) };
                let r = f.read(&mut buf);
                std::mem::forget(f);
                r
            }) {
                Ok(Ok(n)) => { buf.truncate(n); return Ok(buf); }
                Ok(Err(e)) => return Err(TransportError::Io(e)),
                Err(_) => continue,
            }
        }
    }

    pub async fn write_packet(&self, data: &[u8]) -> TransportResult<()> {
        use std::os::fd::AsRawFd;
        use tokio::io::unix::AsyncFd;
        use std::io::Write;

        let raw = self.fd.as_raw_fd();
        let afd = AsyncFd::new(raw).map_err(|e| TransportError::Io(e))?;
        let data = data.to_vec();

        loop {
            let mut guard = afd.writable().await.map_err(|e| TransportError::Io(e))?;
            match guard.try_io(|_| {
                let mut f = unsafe { std::fs::File::from_raw_fd(raw) };
                let r = f.write_all(&data);
                std::mem::forget(f);
                r
            }) {
                Ok(Ok(())) => return Ok(()),
                Ok(Err(e)) => return Err(TransportError::Io(e)),
                Err(_) => continue,
            }
        }
    }

    pub fn set_ip_address(&self, cidr: &str) -> TransportResult<()> {
        let s1 = std::process::Command::new("ip")
            .args(["addr", "add", cidr, "dev", &self.name])
            .status().map_err(|e| TransportError::Io(e))?;
        if !s1.success() {
            return Err(TransportError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("ip addr add failed for {cidr} dev {}", self.name),
            )));
        }
        let s2 = std::process::Command::new("ip")
            .args(["link", "set", &self.name, "up"])
            .status().map_err(|e| TransportError::Io(e))?;
        if !s2.success() {
            return Err(TransportError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("ip link set up failed for {}", self.name),
            )));
        }
        Ok(())
    }

    pub fn enable_ip_forward() -> TransportResult<()> {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .write(true).open("/proc/sys/net/ipv4/ip_forward")
            .map_err(|e| TransportError::Io(e))?;
        f.write_all(b"1").map_err(|e| TransportError::Io(e))?;
        Ok(())
    }

    pub fn setup_nat(subnet: &str) -> TransportResult<()> {
        let s = std::process::Command::new("iptables")
            .args(["-t", "nat", "-A", "POSTROUTING", "-s", subnet, "-j", "MASQUERADE"])
            .status().map_err(|e| TransportError::Io(e))?;
        if !s.success() {
            return Err(TransportError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("iptables NAT setup failed for {subnet}"),
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