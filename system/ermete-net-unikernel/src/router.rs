#![allow(dead_code)]

use smoltcp::wire::{IpAddress, IpCidr, Ipv4Address, Ipv6Address};
use std::collections::HashSet;

/// Policy enforcement mode for Micro-VM traffic isolation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationPolicy {
    /// Micro-VMs can communicate with the host stack but NOT directly with each other
    IsolatedEnclave,
    /// Strict air-gap: Micro-VMs can only respond to explicitly authorized endpoints
    AirGapped,
    /// Permissive development mode
    Promiscuous,
}

pub struct PacketRouter {
    pub policy: IsolationPolicy,
    pub subnet_ipv4: IpCidr,
    pub subnet_ipv6: IpCidr,
    registered_microvms: HashSet<IpAddress>,
}

impl PacketRouter {
    pub fn new(policy: IsolationPolicy) -> Self {
        Self {
            policy,
            subnet_ipv4: IpCidr::new(IpAddress::Ipv4(Ipv4Address::new(10, 0, 2, 0)), 24),
            subnet_ipv6: IpCidr::new(IpAddress::Ipv6(Ipv6Address::new(0xfd00, 0, 0, 0, 0, 0, 0, 0)), 64),
            registered_microvms: HashSet::new(),
        }
    }

    pub fn register_microvm(&mut self, ip: IpAddress) {
        tracing::info!(target: "ermete_net", "Registering isolated Micro-VM node IP: {}", ip);
        self.registered_microvms.insert(ip);
    }

    pub fn unregister_microvm(&mut self, ip: &IpAddress) {
        self.registered_microvms.remove(ip);
    }

    pub fn active_microvm_count(&self) -> usize {
        self.registered_microvms.len()
    }

    /// Zero-Trust security filter: evaluates packet flow between source and destination
    pub fn authorize_flow(&self, src: IpAddress, dst: IpAddress) -> bool {
        match self.policy {
            IsolationPolicy::Promiscuous => true,
            IsolationPolicy::AirGapped => {
                // Air-gapped mode: permit only loopback or host-gateway traffic
                let host_gw_v4 = IpAddress::Ipv4(Ipv4Address::new(10, 0, 2, 1));
                let host_gw_v6 = IpAddress::Ipv6(Ipv6Address::new(0xfd00, 0, 0, 0, 0, 0, 0, 1));
                src == host_gw_v4 || dst == host_gw_v4 || src == host_gw_v6 || dst == host_gw_v6
            }
            IsolationPolicy::IsolatedEnclave => {
                // Prevent Micro-VM spoofing
                if !self.registered_microvms.contains(&src) && !self.is_host_gateway(src) {
                    tracing::warn!(
                        target: "ermete_net",
                        "SECURITY VIOLATION: Dropping spoofed packet from unregistered IP {}",
                        src
                    );
                    return false;
                }

                // If both src and dst are micro-VMs, block direct peer-to-peer unless configured
                if self.registered_microvms.contains(&src) && self.registered_microvms.contains(&dst) {
                    tracing::warn!(
                        target: "ermete_net",
                        "POLICY DENY: Blocked inter-VM direct packet flow from {} -> {}",
                        src,
                        dst
                    );
                    return false;
                }

                true
            }
        }
    }

    fn is_host_gateway(&self, ip: IpAddress) -> bool {
        match ip {
            IpAddress::Ipv4(v4) => v4 == Ipv4Address::new(10, 0, 2, 1),
            IpAddress::Ipv6(v6) => v6 == Ipv6Address::new(0xfd00, 0, 0, 0, 0, 0, 0, 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_trust_policy_enforcement() {
        let mut router = PacketRouter::new(IsolationPolicy::IsolatedEnclave);
        let gw = IpAddress::Ipv4(Ipv4Address::new(10, 0, 2, 1));
        let vm1 = IpAddress::Ipv4(Ipv4Address::new(10, 0, 2, 10));
        let vm2 = IpAddress::Ipv4(Ipv4Address::new(10, 0, 2, 20));
        let spoofed = IpAddress::Ipv4(Ipv4Address::new(10, 0, 2, 99));

        router.register_microvm(vm1);
        router.register_microvm(vm2);

        // Gateway -> VM1 allowed
        assert!(router.authorize_flow(gw, vm1));

        // VM1 -> Gateway allowed
        assert!(router.authorize_flow(vm1, gw));

        // Inter-VM direct flow denied in isolated enclave mode
        assert!(!router.authorize_flow(vm1, vm2));

        // Unregistered/spoofed IP flow denied
        assert!(!router.authorize_flow(spoofed, gw));
    }
}
