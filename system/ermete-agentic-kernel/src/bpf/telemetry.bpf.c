#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>
#include <linux/if_ether.h>
#include <linux/ip.h>
#include <linux/tcp.h>

struct ai_sched_param {
    __u32 pid;
    __u32 target_core;
    __u8 core_type;
    __u8 _pad[3];
    __u32 cpu_weight;
    __u64 slice_us;
    __u32 sched_class;
    __u64 latency_target_us;
    __u32 flags;
};

// FIREWALL_STATS: 0=Passed, 1=Dropped, 2=LAND, 3=TCP Scans, 4=Blocklist, 5=Port
struct {
    __uint(type, BPF_MAP_TYPE_ARRAY);
    __type(key, __u32);
    __type(value, __u64);
    __uint(max_entries, 16);
} FIREWALL_STATS SEC(".maps");

// BLOCKLIST_IPV4
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __type(key, __u32);
    __type(value, __u32);
    __uint(max_entries, 1024);
} BLOCKLIST_IPV4 SEC(".maps");

// CONFIG_FLAGS
struct {
    __uint(type, BPF_MAP_TYPE_ARRAY);
    __type(key, __u32);
    __type(value, __u32);
    __uint(max_entries, 4);
} CONFIG_FLAGS SEC(".maps");

// AI_SCHED_MAP (Shared with sched_ext)
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __type(key, __u32);
    __type(value, struct ai_sched_param);
    __uint(max_entries, 4096);
} AI_SCHED_MAP SEC(".maps");

// ALLOWLIST_IPV4
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __type(key, __u32);
    __type(value, __u32);
    __uint(max_entries, 1024);
} ALLOWLIST_IPV4 SEC(".maps");

static __always_inline void increment_stat(__u32 index) {
    __u64 *val = bpf_map_lookup_elem(&FIREWALL_STATS, &index);
    if (val) {
        __sync_fetch_and_add(val, 1);
    }
}

SEC("xdp")
int xdp_firewall(struct xdp_md *ctx) {
    void *data_end = (void *)(long)ctx->data_end;
    void *data = (void *)(long)ctx->data;

    struct ethhdr *eth = data;
    if (data + sizeof(*eth) > data_end)
        return XDP_PASS;

    if (eth->h_proto != __constant_htons(ETH_P_IP)) {
        increment_stat(0); // Passed
        return XDP_PASS;
    }

    struct iphdr *ip = data + sizeof(*eth);
    if ((void *)ip + sizeof(*ip) > data_end)
        return XDP_PASS;

    __u32 src_ip = ip->saddr;

    // Zero-Trust mode check
    __u32 zt_idx = 0;
    __u32 *zt_mode = bpf_map_lookup_elem(&CONFIG_FLAGS, &zt_idx);
    if (zt_mode && *zt_mode == 1) {
        if (!bpf_map_lookup_elem(&ALLOWLIST_IPV4, &src_ip)) {
            increment_stat(1); // Dropped
            return XDP_DROP;
        }
    }

    // Check Blocklist
    if (bpf_map_lookup_elem(&BLOCKLIST_IPV4, &src_ip)) {
        increment_stat(1); // Dropped
        increment_stat(4); // Blocklist drops
        return XDP_DROP;
    }

    // LAND attack check
    if (ip->saddr == ip->daddr) {
        increment_stat(1); // Dropped
        increment_stat(2); // LAND attacks
        return XDP_DROP;
    }

    if (ip->protocol == IPPROTO_TCP) {
        struct tcphdr *tcp = (void *)ip + sizeof(*ip);
        if ((void *)tcp + sizeof(*tcp) > data_end)
            return XDP_PASS;

        // Port checks (e.g., block 23 Telnet)
        if (tcp->dest == __constant_htons(23)) {
            increment_stat(1); // Dropped
            increment_stat(5); // Unauthorized port
            return XDP_DROP;
        }

        // SYN flood basic scan detection
        if (tcp->syn && !tcp->ack) {
            increment_stat(3); // TCP Scans
        }
    }

    increment_stat(0); // Passed
    return XDP_PASS;
}

char _license[] SEC("license") = "GPL";
