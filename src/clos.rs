use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct ClosTopology {
    pub num_hosts: u32,
    pub num_leafs: u32,
    pub num_spines: u32,
    pub ports_per_switch: u32,
    pub links_per_host: u32,
    pub hosts_per_leaf: u32,
    pub links_per_leaf_spine_pair: u32,
    pub leaf_south_ports_used: u32,
    pub leaf_north_ports_used: u32,
    pub spine_ports_used: u32,
    pub link_bandwidth: u32,
    pub links: Vec<[u32; 3]>, // [src, dst, bandwidth]
}

impl ClosTopology {
    pub fn total_switches(&self) -> u32 {
        self.num_leafs + self.num_spines
    }

    pub fn host_id_range(&self) -> (u32, u32) {
        (0, self.num_hosts - 1)
    }

    pub fn leaf_id_range(&self) -> (u32, u32) {
        let start = self.num_hosts;
        (start, start + self.num_leafs - 1)
    }

    pub fn spine_id_range(&self) -> (u32, u32) {
        let start = self.num_hosts + self.num_leafs;
        (start, start + self.num_spines - 1)
    }

    pub fn summary(&self) -> String {
        let (h0, h1) = self.host_id_range();
        let (l0, l1) = self.leaf_id_range();
        let (s0, s1) = self.spine_id_range();
        let half = self.ports_per_switch / 2;
        format!(
            "=== 2-Layer CLOS Topology ===\n\
             Hosts:          {}\n\
             \x20\x20IDs:          [{}, {}]\n\
             \x20\x20Links/host:   {} x {}G (aggregated: {}G)\n\
             Leaf switches:  {}\n\
             \x20\x20IDs:          [{}, {}]\n\
             \x20\x20South ports:  {}/{} used\n\
             \x20\x20North ports:  {}/{} used\n\
             \x20\x20Total ports:  {}/{} used\n\
             Spine switches: {}\n\
             \x20\x20IDs:          [{}, {}]\n\
             \x20\x20Ports used:   {}/{} used\n\
             Total switches: {}\n\
             Total links:    {}",
            self.num_hosts,
            h0,
            h1,
            self.links_per_host,
            self.link_bandwidth,
            self.links_per_host * self.link_bandwidth,
            self.num_leafs,
            l0,
            l1,
            self.leaf_south_ports_used,
            half,
            self.leaf_north_ports_used,
            half,
            self.leaf_south_ports_used + self.leaf_north_ports_used,
            self.ports_per_switch,
            self.num_spines,
            s0,
            s1,
            self.spine_ports_used,
            self.ports_per_switch,
            self.total_switches(),
            self.links.len()
        )
    }

    pub fn to_json(&self) -> Vec<[u32; 3]> {
        self.links.clone()
    }

    pub fn write_json(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self.to_json())?;
        fs::write(path, json)
    }
}

pub fn generate(
    switch_throughput: u32,
    nic_throughput: u32,
    link_bandwidth: u32,
    num_hosts: u32,
) -> Result<ClosTopology, String> {
    validate_inputs(switch_throughput, nic_throughput, link_bandwidth, num_hosts)?;

    let ports_per_switch = switch_throughput / link_bandwidth;
    let links_per_host = nic_throughput / link_bandwidth;
    let half_ports = ports_per_switch / 2;

    let hosts_per_leaf = half_ports / links_per_host;
    let num_leafs = num_hosts / hosts_per_leaf;
    let leaf_south_ports_used = hosts_per_leaf * links_per_host;

    let max_links_per_pair = ports_per_switch / num_leafs;
    if max_links_per_pair < 1 {
        return Err(format!(
            "Cannot build topology: {} leafs require spines with >= {} ports, but switches only have {} ports.",
            num_leafs, num_leafs, ports_per_switch
        ));
    }

    let links_per_leaf_spine_pair = largest_divisor_leq(half_ports, max_links_per_pair);
    let num_spines = half_ports / links_per_leaf_spine_pair;
    let spine_ports_used = num_leafs * links_per_leaf_spine_pair;
    let leaf_north_ports_used = num_spines * links_per_leaf_spine_pair;

    let mut links = Vec::new();
    let leaf_id_start = num_hosts;
    let spine_id_start = num_hosts + num_leafs;

    // Host-to-leaf
    let agg_host_bw = links_per_host * link_bandwidth;
    for leaf_idx in 0..num_leafs {
        let leaf_id = leaf_id_start + leaf_idx;
        for h in 0..hosts_per_leaf {
            let host_id = leaf_idx * hosts_per_leaf + h;
            links.push([host_id, leaf_id, agg_host_bw]);
        }
    }

    // Leaf-to-spine
    let agg_uplink_bw = links_per_leaf_spine_pair * link_bandwidth;
    for leaf_idx in 0..num_leafs {
        let leaf_id = leaf_id_start + leaf_idx;
        for spine_idx in 0..num_spines {
            let spine_id = spine_id_start + spine_idx;
            links.push([leaf_id, spine_id, agg_uplink_bw]);
        }
    }

    Ok(ClosTopology {
        num_hosts,
        num_leafs,
        num_spines,
        ports_per_switch,
        links_per_host,
        hosts_per_leaf,
        links_per_leaf_spine_pair,
        leaf_south_ports_used,
        leaf_north_ports_used,
        spine_ports_used,
        link_bandwidth,
        links,
    })
}

fn validate_inputs(
    switch_throughput: u32,
    nic_throughput: u32,
    link_bandwidth: u32,
    num_hosts: u32,
) -> Result<(), String> {
    if switch_throughput == 0 || nic_throughput == 0 || link_bandwidth == 0 || num_hosts == 0 {
        return Err("All throughput/bandwidth values and num_hosts must be positive".to_string());
    }
    if !switch_throughput.is_multiple_of(link_bandwidth) {
        return Err(format!(
            "switch_throughput ({}) must be divisible by link_bandwidth ({})",
            switch_throughput, link_bandwidth
        ));
    }
    if !nic_throughput.is_multiple_of(link_bandwidth) {
        return Err(format!(
            "nic_throughput ({}) must be divisible by link_bandwidth ({})",
            nic_throughput, link_bandwidth
        ));
    }

    let ports_per_switch = switch_throughput / link_bandwidth;
    if !ports_per_switch.is_multiple_of(2) {
        return Err(format!(
            "ports_per_switch ({}) must be even for non-oversubscribed leaf-spine split",
            ports_per_switch
        ));
    }

    let links_per_host = nic_throughput / link_bandwidth;
    let half_ports = ports_per_switch / 2;
    if !half_ports.is_multiple_of(links_per_host) {
        return Err(format!(
            "Half the switch ports ({}) must be divisible by links_per_host ({})",
            half_ports, links_per_host
        ));
    }

    let hosts_per_leaf = half_ports / links_per_host;
    if !num_hosts.is_multiple_of(hosts_per_leaf) {
        return Err(format!(
            "num_hosts ({}) must be divisible by hosts_per_leaf ({})",
            num_hosts, hosts_per_leaf
        ));
    }
    Ok(())
}

fn largest_divisor_leq(n: u32, cap: u32) -> u32 {
    for d in (1..=std::cmp::min(cap, n)).rev() {
        if n % d == 0 {
            return d;
        }
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clos_basic_128() {
        let topo = generate(6400, 800, 200, 128).unwrap();
        assert_eq!(topo.num_hosts, 128);
        assert_eq!(topo.total_switches(), 48);
        assert_eq!(topo.links.len(), 640);
        assert_eq!(topo.num_leafs, 32);
        assert_eq!(topo.num_spines, 16);
        let (h0, h1) = topo.host_id_range();
        assert_eq!((h0, h1), (0, 127));
    }

    #[test]
    fn test_clos_small_cases() {
        for &n in &[4, 8, 16, 32, 64] {
            let topo = generate(6400, 800, 200, n).expect(&format!("failed for {} hosts", n));
            assert_eq!(topo.num_hosts, n);
            assert!(topo.links.len() > 0);
            assert!(topo.num_leafs > 0);
            assert!(topo.num_spines > 0);
        }
    }

    #[test]
    fn test_clos_validation_errors() {
        assert!(generate(6400, 800, 200, 0).is_err());
        assert!(generate(6400, 800, 199, 64).is_err()); // not divisible
        assert!(generate(6401, 800, 200, 64).is_err()); // not divisible
    }

    #[test]
    fn test_write_json() {
        use tempfile::tempdir;
        let topo = generate(6400, 800, 200, 8).unwrap();
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.json");
        topo.write_json(&path).unwrap();
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        // Pretty-printed JSON starts with [
        assert!(content.trim_start().starts_with('['));
        assert!(content.contains("0"));
    }
}
