#![allow(dead_code)]

use anyhow::{Context, Result};
use smoltcp::phy::{Device, DeviceCapabilities, Loopback, Medium, RxToken, TunTapInterface, TxToken};
use smoltcp::time::Instant;
use std::os::unix::io::AsRawFd;

/// Network backend selector for userspace TCP/IP bypass stack.
pub enum NetworkBackend {
    /// Linux TUN/TAP tap interface (Ethernet layer bypass)
    TunTap(TunTapInterface),
    /// Synthetic high-throughput loopback device for micro-VM IPC & testing
    Loopback(Loopback),
}

pub struct DeviceManager {
    interface_name: String,
    backend: NetworkBackend,
    medium: Medium,
}

impl DeviceManager {
    /// Attach to a host TUN/TAP interface (e.g. "tap-ermete0" or "tun-ermete0")
    pub fn new_tuntap(name: &str, medium: Medium) -> Result<Self> {
        let tuntap = TunTapInterface::new(name, medium)
            .with_context(|| format!("Failed to bind TUN/TAP device interface '{}'", name))?;

        tracing::info!(
            target: "ermete_net",
            "Bound smoltcp stack to TUN/TAP interface '{}' (FD: {})",
            name,
            tuntap.as_raw_fd()
        );

        Ok(Self {
            interface_name: name.to_string(),
            backend: NetworkBackend::TunTap(tuntap),
            medium,
        })
    }

    /// Create an isolated loopback device for zero-cost Micro-VM test rings
    pub fn new_loopback(medium: Medium) -> Self {
        let loopback = Loopback::new(medium);
        tracing::info!(target: "ermete_net", "Initialized isolated Loopback device backend");

        Self {
            interface_name: "lo-ermete".to_string(),
            backend: NetworkBackend::Loopback(loopback),
            medium,
        }
    }

    pub fn interface_name(&self) -> &str {
        &self.interface_name
    }

    pub fn medium(&self) -> Medium {
        self.medium
    }
}

pub enum DeviceRxToken<'a> {
    TunTap(<TunTapInterface as Device>::RxToken<'a>),
    Loopback(<Loopback as Device>::RxToken<'a>),
}

pub enum DeviceTxToken<'a> {
    TunTap(<TunTapInterface as Device>::TxToken<'a>),
    Loopback(<Loopback as Device>::TxToken<'a>),
}

impl<'a> RxToken for DeviceRxToken<'a> {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        match self {
            DeviceRxToken::TunTap(tok) => tok.consume(f),
            DeviceRxToken::Loopback(tok) => tok.consume(f),
        }
    }
}

impl<'a> TxToken for DeviceTxToken<'a> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        match self {
            DeviceTxToken::TunTap(tok) => tok.consume(len, f),
            DeviceTxToken::Loopback(tok) => tok.consume(len, f),
        }
    }
}

impl Device for DeviceManager {
    type RxToken<'a> = DeviceRxToken<'a> where Self: 'a;
    type TxToken<'a> = DeviceTxToken<'a> where Self: 'a;

    fn receive(&mut self, timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        match &mut self.backend {
            NetworkBackend::TunTap(dev) => dev
                .receive(timestamp)
                .map(|(rx, tx)| (DeviceRxToken::TunTap(rx), DeviceTxToken::TunTap(tx))),
            NetworkBackend::Loopback(dev) => dev
                .receive(timestamp)
                .map(|(rx, tx)| (DeviceRxToken::Loopback(rx), DeviceTxToken::Loopback(tx))),
        }
    }

    fn transmit(&mut self, timestamp: Instant) -> Option<Self::TxToken<'_>> {
        match &mut self.backend {
            NetworkBackend::TunTap(dev) => dev.transmit(timestamp).map(DeviceTxToken::TunTap),
            NetworkBackend::Loopback(dev) => dev.transmit(timestamp).map(DeviceTxToken::Loopback),
        }
    }

    fn capabilities(&self) -> DeviceCapabilities {
        match &self.backend {
            NetworkBackend::TunTap(dev) => dev.capabilities(),
            NetworkBackend::Loopback(dev) => dev.capabilities(),
        }
    }
}
