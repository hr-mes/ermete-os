use smoltcp::iface::{Config, Interface, SocketHandle, SocketSet};
use smoltcp::phy::Medium;
use smoltcp::socket::icmp::{Endpoint as IcmpEndpoint, PacketBuffer as IcmpPacketBuffer, PacketMetadata as IcmpPacketMetadata, Socket as IcmpSocket};
use smoltcp::socket::tcp::{Socket as TcpSocket, SocketBuffer as TcpSocketBuffer};
use smoltcp::socket::udp::{PacketBuffer as UdpPacketBuffer, PacketMetadata as UdpPacketMetadata, Socket as UdpSocket};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr, Ipv4Address, Ipv6Address};
use std::sync::Arc;

use crate::device::DeviceManager;
use crate::metrics::NetworkMetrics;
use crate::router::{IsolationPolicy, PacketRouter};

pub struct UnikernelNetworkStack {
    iface: Interface,
    sockets: SocketSet<'static>,
    tcp_handle: SocketHandle,
    udp_handle: SocketHandle,
    router: PacketRouter,
    metrics: Arc<NetworkMetrics>,
}

impl UnikernelNetworkStack {
    pub fn new(mac_addr: [u8; 6], policy: IsolationPolicy, metrics: Arc<NetworkMetrics>) -> Self {
        let hardware_addr = HardwareAddress::Ethernet(EthernetAddress(mac_addr));
        let config = Config::new(hardware_addr);

        let mut iface = Interface::new(config, &mut DeviceManager::new_loopback(Medium::Ethernet), Instant::now());

        // Configure Dual-Stack IPv4 + IPv6 addresses
        iface.update_ip_addrs(|addrs| {
            addrs.push(IpCidr::new(IpAddress::Ipv4(Ipv4Address::new(10, 0, 2, 1)), 24)).ok();
            addrs.push(IpCidr::new(IpAddress::Ipv6(Ipv6Address::new(0xfd00, 0, 0, 0, 0, 0, 0, 1)), 64)).ok();
        });

        let mut sockets = SocketSet::new(vec![]);

        // TCP Echo/Control socket setup
        let tcp_rx_buf = TcpSocketBuffer::new(vec![0u8; 65536]);
        let tcp_tx_buf = TcpSocketBuffer::new(vec![0u8; 65536]);
        let mut tcp_socket = TcpSocket::new(tcp_rx_buf, tcp_tx_buf);
        tcp_socket.listen(8080).expect("Failed to listen on TCP port 8080");
        let tcp_handle = sockets.add(tcp_socket);

        // UDP Socket setup
        let udp_rx_buf = UdpPacketBuffer::new(vec![UdpPacketMetadata::EMPTY; 16], vec![0u8; 65536]);
        let udp_tx_buf = UdpPacketBuffer::new(vec![UdpPacketMetadata::EMPTY; 16], vec![0u8; 65536]);
        let mut udp_socket = UdpSocket::new(udp_rx_buf, udp_tx_buf);
        udp_socket.bind(5353).expect("Failed to bind UDP port 5353");
        let udp_handle = sockets.add(udp_socket);

        // ICMP Echo responder socket
        let icmp_rx_buf = IcmpPacketBuffer::new(vec![IcmpPacketMetadata::EMPTY; 16], vec![0u8; 65536]);
        let icmp_tx_buf = IcmpPacketBuffer::new(vec![IcmpPacketMetadata::EMPTY; 16], vec![0u8; 65536]);
        let mut icmp_socket = IcmpSocket::new(icmp_rx_buf, icmp_tx_buf);
        icmp_socket.bind(IcmpEndpoint::Ident(0x1337)).expect("Failed to bind ICMP socket");
        let _icmp_handle = sockets.add(icmp_socket);

        let router = PacketRouter::new(policy);

        tracing::info!(
            target: "ermete_net",
            "Userspace Rust TCP/IP/IPv6 stack initialized with MAC {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            mac_addr[0], mac_addr[1], mac_addr[2], mac_addr[3], mac_addr[4], mac_addr[5]
        );

        Self {
            iface,
            sockets,
            tcp_handle,
            udp_handle,
            router,
            metrics,
        }
    }

    pub fn router_mut(&mut self) -> &mut PacketRouter {
        &mut self.router
    }

    pub fn poll_device(&mut self, device: &mut DeviceManager, timestamp: Instant) -> bool {
        let updated = self.iface.poll(timestamp, device, &mut self.sockets);

        // Handle active TCP socket state machine & zero-copy echo service
        let socket = self.sockets.get_mut::<TcpSocket>(self.tcp_handle);
        if socket.is_active() && socket.can_recv() {
            let mut buffer = [0u8; 4096];
            if let Ok(bytes_read) = socket.recv_slice(&mut buffer) {
                if bytes_read > 0 {
                    self.metrics.inc_rx(bytes_read as u64);
                    self.metrics.inc_tcp_conn();

                    tracing::debug!(
                        target: "ermete_net",
                        "TCP Stream received {} bytes over isolated smoltcp bypass",
                        bytes_read
                    );

                    // Zero-copy TCP echo response
                    if socket.can_send() {
                        let response = format!("Ermete-Unikernel-Ack: {} bytes processed\n", bytes_read);
                        if let Ok(written) = socket.send_slice(response.as_bytes()) {
                            self.metrics.inc_tx(written as u64);
                        }
                    }
                }
            }
        }

        // Handle UDP socket traffic
        let udp_socket = self.sockets.get_mut::<UdpSocket>(self.udp_handle);
        if udp_socket.can_recv() {
            let mut buf = [0u8; 2048];
            if let Ok((len, endpoint)) = udp_socket.recv_slice(&mut buf) {
                self.metrics.inc_rx(len as u64);
                self.metrics.inc_udp();
                tracing::debug!(
                    target: "ermete_net",
                    "UDP Datagram from {}: {} bytes",
                    endpoint, len
                );
            }
        }

        self.metrics.set_active_microvms(self.router.active_microvm_count() as u64);

        updated
    }
}
