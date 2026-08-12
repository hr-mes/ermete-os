#![no_std]
#![no_main]
#![allow(unsafe_code)]

use aya_ebpf::{
    bindings::xdp_action,
    macros::{map, xdp},
    maps::{Array, HashMap},
    programs::XdpContext,
};
use aya_log_ebpf::warn;
use core::mem;
use network_types::{
    eth::{EthHdr, EtherType},
    ip::{IpProto, Ipv4Hdr, Ipv6Hdr},
    tcp::TcpHdr,
    udp::UdpHdr,
};

// Firewall Statistics Counter Map Indices
pub const STAT_PASSED: u32 = 0;
pub const STAT_DROP_INVALID_HDR: u32 = 1;
pub const STAT_DROP_LAND_ATTACK: u32 = 2;
pub const STAT_DROP_ANOMALOUS_FLAGS: u32 = 3;
pub const STAT_DROP_BLOCKLIST_IP: u32 = 4;
pub const STAT_DROP_UNAUTHORIZED_PORT: u32 = 5;

// eBPF Maps
#[map]
static BLOCKLIST_IPV4: HashMap<u32, u32> = HashMap::with_max_entries(1024, 0);

#[map]
static BLOCKLIST_IPV6: HashMap<[u8; 16], u32> = HashMap::with_max_entries(1024, 0);

#[map]
static ALLOWED_PORTS: HashMap<u16, u32> = HashMap::with_max_entries(256, 0);

#[map]
static FIREWALL_STATS: Array<u64> = Array::with_max_entries(8, 0);

#[map]
static CONFIG_FLAGS: Array<u32> = Array::with_max_entries(4, 0);
// CONFIG_FLAGS[0]: zero_trust_enabled (1 = drop unknown ports, 0 = pass unknown ports)

#[inline(always)]
fn increment_stat(index: u32) {
    if let Some(ptr) = FIREWALL_STATS.get_ptr_mut(index) {
        // SAFETY: Pointer is valid because get_ptr_mut returns a checked pointer.
        unsafe {
            *ptr += 1;
        }
    }
}

#[inline(always)]
fn is_zero_trust_enabled() -> bool {
    if let Some(val) = CONFIG_FLAGS.get(0) {
        *val != 0
    } else {
        false
    }
}

#[inline(always)]
fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Result<*const T, ()> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = mem::size_of::<T>();

    if start.checked_add(offset).and_then(|p| p.checked_add(len)).is_none_or(|ptr_end| ptr_end > end) {
        return Err(());
    }

    Ok((start + offset) as *const T)
}

#[xdp]
pub fn xdp_firewall(ctx: XdpContext) -> u32 {
    match try_xdp_firewall(&ctx) {
        Ok(action) => action,
        Err(_) => {
            increment_stat(STAT_DROP_INVALID_HDR);
            xdp_action::XDP_DROP
        }
    }
}

fn try_xdp_firewall(ctx: &XdpContext) -> Result<u32, ()> {
    let data = ctx.data();
    let data_end = ctx.data_end();

    if data_end < data + mem::size_of::<EthHdr>() {
        return Ok(xdp_action::XDP_ABORTED);
    }

    let ethhdr: *const EthHdr = ptr_at(ctx, 0)?;

    // SAFETY: Memory bounds verified by prior checks or eBPF verifier
    match unsafe { (*ethhdr).ether_type } {
        EtherType::Ipv4 => process_ipv4(ctx),
        EtherType::Ipv6 => process_ipv6(ctx),
        EtherType::Arp => {
            increment_stat(STAT_PASSED);
            Ok(xdp_action::XDP_PASS)
        }
        _ => {
            if is_zero_trust_enabled() {
                increment_stat(STAT_DROP_INVALID_HDR);
                Ok(xdp_action::XDP_DROP)
            } else {
                increment_stat(STAT_PASSED);
                Ok(xdp_action::XDP_PASS)
            }
        }
    }
}

fn process_ipv4(ctx: &XdpContext) -> Result<u32, ()> {
    let data = ctx.data();
    let data_end = ctx.data_end();

    if data_end < data + EthHdr::LEN + mem::size_of::<Ipv4Hdr>() {
        return Ok(xdp_action::XDP_ABORTED);
    }

    let ipv4hdr: *const Ipv4Hdr = ptr_at(ctx, EthHdr::LEN)?;

    // Validate IPv4 Header Length (IHL)
    // SAFETY: Memory bounds verified by prior checks or eBPF verifier
    let ihl = unsafe { ((*ipv4hdr).ihl() & 0x0F) as usize * 4 };
    if ihl < Ipv4Hdr::LEN {
        increment_stat(STAT_DROP_INVALID_HDR);
        warn!(ctx, "XDP_DROP: Invalid IPv4 IHL < 20 bytes");
        return Ok(xdp_action::XDP_DROP);
    }

    if data_end < data + EthHdr::LEN + ihl {
        increment_stat(STAT_DROP_INVALID_HDR);
        warn!(ctx, "XDP_DROP: Packet smaller than IPv4 IHL");
        return Ok(xdp_action::XDP_ABORTED);
    }

    // SAFETY: Memory bounds verified by prior checks or eBPF verifier
    let src_addr = unsafe { (*ipv4hdr).src_addr };
    // SAFETY: Memory bounds verified by prior checks or eBPF verifier
    let dst_addr = unsafe { (*ipv4hdr).dst_addr };

    // 1. Check Land Attack (src IP == dst IP)
    if src_addr == dst_addr && src_addr != 0 {
        increment_stat(STAT_DROP_LAND_ATTACK);
        warn!(ctx, "XDP_DROP: Land Attack detected (src == dst IP)");
        return Ok(xdp_action::XDP_DROP);
    }

    // 2. Check IPv4 Blocklist
    // SAFETY: Memory bounds verified by prior checks or eBPF verifier
    if unsafe { BLOCKLIST_IPV4.get(&src_addr) }.is_some() {
        increment_stat(STAT_DROP_BLOCKLIST_IP);
        warn!(ctx, "XDP_DROP: Source IPv4 in blocklist");
        return Ok(xdp_action::XDP_DROP);
    }

    // SAFETY: Memory bounds verified by prior checks or eBPF verifier
    let ip_proto = unsafe { (*ipv4hdr).proto };
    let transport_offset = EthHdr::LEN + ihl;

    match ip_proto {
        IpProto::Tcp => process_tcp(ctx, transport_offset),
        IpProto::Udp => process_udp(ctx, transport_offset),
        IpProto::Icmp => {
            increment_stat(STAT_PASSED);
            Ok(xdp_action::XDP_PASS)
        }
        _ => {
            if is_zero_trust_enabled() {
                increment_stat(STAT_DROP_INVALID_HDR);
                Ok(xdp_action::XDP_DROP)
            } else {
                increment_stat(STAT_PASSED);
                Ok(xdp_action::XDP_PASS)
            }
        }
    }
}

fn process_ipv6(ctx: &XdpContext) -> Result<u32, ()> {
    let data = ctx.data();
    let data_end = ctx.data_end();

    if data_end < data + EthHdr::LEN + mem::size_of::<Ipv6Hdr>() {
        return Ok(xdp_action::XDP_ABORTED);
    }

    let ipv6hdr: *const Ipv6Hdr = ptr_at(ctx, EthHdr::LEN)?;
    // SAFETY: Memory bounds verified by prior checks or eBPF verifier
    let src_bytes = unsafe { (*ipv6hdr).src_addr.in6_u.u6_addr8 };
    // SAFETY: Memory bounds verified by prior checks or eBPF verifier
    let dst_bytes = unsafe { (*ipv6hdr).dst_addr.in6_u.u6_addr8 };

    // 1. Check Land Attack (src IP == dst IP)
    if src_bytes == dst_bytes {
        increment_stat(STAT_DROP_LAND_ATTACK);
        warn!(ctx, "XDP_DROP: IPv6 Land Attack detected");
        return Ok(xdp_action::XDP_DROP);
    }

    // 2. Check IPv6 Blocklist
    // SAFETY: Memory bounds verified by prior checks or eBPF verifier
    if unsafe { BLOCKLIST_IPV6.get(&src_bytes) }.is_some() {
        increment_stat(STAT_DROP_BLOCKLIST_IP);
        warn!(ctx, "XDP_DROP: Source IPv6 in blocklist");
        return Ok(xdp_action::XDP_DROP);
    }

    let transport_offset = EthHdr::LEN + Ipv6Hdr::LEN;
    // SAFETY: Memory bounds verified by prior checks or eBPF verifier
    let next_hdr = unsafe { (*ipv6hdr).next_hdr };

    match next_hdr {
        IpProto::Tcp => process_tcp(ctx, transport_offset),
        IpProto::Udp => process_udp(ctx, transport_offset),
        IpProto::Ipv6Icmp => {
            increment_stat(STAT_PASSED);
            Ok(xdp_action::XDP_PASS)
        }
        _ => {
            if is_zero_trust_enabled() {
                increment_stat(STAT_DROP_INVALID_HDR);
                Ok(xdp_action::XDP_DROP)
            } else {
                increment_stat(STAT_PASSED);
                Ok(xdp_action::XDP_PASS)
            }
        }
    }
}

fn process_tcp(ctx: &XdpContext, offset: usize) -> Result<u32, ()> {
    let data = ctx.data();
    let data_end = ctx.data_end();

    if data_end < data + offset + mem::size_of::<TcpHdr>() {
        return Ok(xdp_action::XDP_ABORTED);
    }

    let tcphdr: *const TcpHdr = ptr_at(ctx, offset)?;
    // SAFETY: Memory bounds verified by prior checks or eBPF verifier
    let dest_port = u16::from_be(unsafe { (*tcphdr).dest });

    // SAFETY: Memory bounds verified by prior checks or eBPF verifier
    let fin = unsafe { (*tcphdr).fin() == 1 };
    // SAFETY: Memory bounds verified by prior checks or eBPF verifier
    let syn = unsafe { (*tcphdr).syn() == 1 };
    // SAFETY: Memory bounds verified by prior checks or eBPF verifier
    let rst = unsafe { (*tcphdr).rst() == 1 };
    // SAFETY: Memory bounds verified by prior checks or eBPF verifier
    let psh = unsafe { (*tcphdr).psh() == 1 };
    // SAFETY: Memory bounds verified by prior checks or eBPF verifier
    let ack = unsafe { (*tcphdr).ack() == 1 };
    // SAFETY: Memory bounds verified by prior checks or eBPF verifier
    let urg = unsafe { (*tcphdr).urg() == 1 };

    // Detect TCP Scan Anomalies:
    // 1. NULL Scan: fin=0, syn=0, rst=0, psh=0, ack=0, urg=0
    let is_null_scan = !fin && !syn && !rst && !psh && !ack && !urg;
    // 2. XMAS Scan: fin=1, psh=1, urg=1
    let is_xmas_scan = fin && psh && urg;
    // 3. SYN-FIN Scan: syn=1 && fin=1
    let is_syn_fin_scan = syn && fin;
    // 4. SYN-RST Scan: syn=1 && rst=1
    let is_syn_rst_scan = syn && rst;

    if is_null_scan || is_xmas_scan || is_syn_fin_scan || is_syn_rst_scan {
        increment_stat(STAT_DROP_ANOMALOUS_FLAGS);
        warn!(ctx, "XDP_DROP: Anomalous TCP flags detected (Scan attempt)");
        return Ok(xdp_action::XDP_DROP);
    }

    // Zero-Trust Port Authorization Check
    if is_zero_trust_enabled() {
        // SAFETY: Memory bounds verified by prior checks or eBPF verifier
        if unsafe { ALLOWED_PORTS.get(&dest_port) }.is_none() {
            increment_stat(STAT_DROP_UNAUTHORIZED_PORT);
            warn!(ctx, "XDP_DROP: Unauthorized TCP destination port: {}", dest_port);
            return Ok(xdp_action::XDP_DROP);
        }
    }

    increment_stat(STAT_PASSED);
    Ok(xdp_action::XDP_PASS)
}

fn process_udp(ctx: &XdpContext, offset: usize) -> Result<u32, ()> {
    let data = ctx.data();
    let data_end = ctx.data_end();

    if data_end < data + offset + mem::size_of::<UdpHdr>() {
        return Ok(xdp_action::XDP_ABORTED);
    }

    let udphdr: *const UdpHdr = ptr_at(ctx, offset)?;
    // SAFETY: Memory bounds verified by prior checks or eBPF verifier
    let dest_port = u16::from_be(unsafe { (*udphdr).dest });

    // Zero-Trust Port Authorization Check
    if is_zero_trust_enabled() {
        // SAFETY: Memory bounds verified by prior checks or eBPF verifier
        if unsafe { ALLOWED_PORTS.get(&dest_port) }.is_none() {
            increment_stat(STAT_DROP_UNAUTHORIZED_PORT);
            warn!(ctx, "XDP_DROP: Unauthorized UDP destination port: {}", dest_port);
            return Ok(xdp_action::XDP_DROP);
        }
    }

    increment_stat(STAT_PASSED);
    Ok(xdp_action::XDP_PASS)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // SAFETY: Panic in eBPF must halt execution.
    unsafe { core::hint::unreachable_unchecked() }
}
