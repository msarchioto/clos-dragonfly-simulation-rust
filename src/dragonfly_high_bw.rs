use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct DragonflyTopology {
    pub num_hosts: u32,
    pub routers_per_group: u32,      // a
    pub num_groups: u32,             // g
    pub hosts_per_router: u32,       // p
    pub global_links_per_router: u32, // h
    pub ports_per_switch: u32,       // k
    pub links_per_host: u32,
    pub link_bandwidth: u32,
    pub links: Vec<[u32; 3]>,
}

impl DragonflyTopology {
    pub fn total_routers(&self) -> u32 {
        self.routers_per_group * self.num_groups
    }

    pub fn host_id_range(&self) -> (u32, u32) {
        (0, self.num_hosts - 1)
    }

    pub fn router_id_range(&self) -> (u32, u32) {
        let start = self.num_hosts;
        (start, start + self.total_routers() - 1)
    }

    pub fn router_ports_used(&self) -> u32 {
        let terminal = self.hosts_per_router * self.links_per_host;
        let local = self.routers_per_group - 1;
        let global = self.global_links_per_router;
        terminal + local + global
    }

    pub fn summary(&self) -> String {
        let a = self.routers_per_group;
        let p = self.hosts_per_router;
        let h = self.global_links_per_router;
        let g = self.num_groups;
        let terminal_ports = p * self.links_per_host;
        let local_ports = a - 1;
        let host_bw = self.links_per_host * self.link_bandwidth;
        let (h0, h1) = self.host_id_range();
        let (r0, r1) = self.router_id_range();

        format!(
            "=== Dragonfly High-BW Topology ===\n\
             Hosts:              {}\n\
             \x20\x20IDs:              [{}, {}]\n\
             \x20\x20Links/host:       {} x {}G (aggregated: {}G)\n\
             Routers:            {}  (a={}, g={})\n\
             \x20\x20IDs:              [{}, {}]\n\
             \x20\x20Terminal ports:   {} (p={}, links_per_host={})\n\
             \x20\x20Local ports:      {} (a-1={})\n\
             \x20\x20Global ports:     {}\n\
             \x20\x20Ports used:       {}/{}\n\
             Groups:             {}\n\
             \x20\x20Routers/group:    {}\n\
             \x20\x20Hosts/group:      {}\n\
             \x20\x20Intra-group links:{} per group\n\
             Total links:        {}",
            self.num_hosts,
            h0, h1,
            self.links_per_host, self.link_bandwidth, host_bw,
            self.total_routers(), a, g,
            r0, r1,
            terminal_ports, p, self.links_per_host,
            local_ports, a-1,
            h,
            self.router_ports_used(), self.ports_per_switch,
            g, a, a * p,
            a * (a - 1) / 2,
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
    router_budget_factor: f64,
) -> Result<DragonflyTopology, String> {
    validate_inputs(switch_throughput, nic_throughput, link_bandwidth, num_hosts)?;

    let ports = switch_throughput / link_bandwidth;
    let lph = nic_throughput / link_bandwidth;

    let (a, h, p, g) = find_best_config(ports, lph, num_hosts, router_budget_factor);

    let actual_hosts = num_hosts;
    let router_id_start = actual_hosts;

    let mut links = Vec::new();

    // 1. Host-to-router (same as standard)
    let agg_host_bw = lph * link_bandwidth;
    let total_routers = a * g;
    let mut router_host_counts = vec![0u32; total_routers as usize];
    let mut rr_order = Vec::new();
    for router_in_group in 0..a {
        for grp in 0..g {
            rr_order.push(grp * a + router_in_group);
        }
    }
    for i in 0..actual_hosts {
        let idx = (i % total_routers) as usize;
        router_host_counts[rr_order[idx] as usize] += 1;
    }

    let mut host_id = 0u32;
    for (ro, cnt) in router_host_counts.iter().enumerate() {
        if *cnt > p {
            return Err(format!("Internal error: router {} over capacity", ro));
        }
        let rid = router_id_start + ro as u32;
        for _ in 0..*cnt {
            links.push([host_id, rid, agg_host_bw]);
            host_id += 1;
        }
    }

    // 2. Intra-group
    for grp in 0..g {
        for i in 0..a {
            for j in (i + 1)..a {
                let src = router_id_start + grp * a + i;
                let dst = router_id_start + grp * a + j;
                links.push([src, dst, link_bandwidth]);
            }
        }
    }

    // 3. Global (same wire)
    let global_pairs = wire_global_links(a, g, h);
    for (src_off, dst_off) in global_pairs {
        let src = router_id_start + src_off;
        let dst = router_id_start + dst_off;
        links.push([src, dst, link_bandwidth]);
    }

    links.sort();

    Ok(DragonflyTopology {
        num_hosts: actual_hosts,
        routers_per_group: a,
        num_groups: g,
        hosts_per_router: p,
        global_links_per_router: h,
        ports_per_switch: ports,
        links_per_host: lph,
        link_bandwidth,
        links,
    })
}

fn validate_inputs(s: u32, n: u32, l: u32, h: u32) -> Result<(), String> {
    if s == 0 || n == 0 || l == 0 || h == 0 {
        return Err("All values must be positive".to_string());
    }
    if s % l != 0 {
        return Err("switch_throughput must be divisible by link_bandwidth".to_string());
    }
    if n % l != 0 {
        return Err("nic_throughput must be divisible by link_bandwidth".to_string());
    }
    Ok(())
}

fn find_best_config(k: u32, lph: u32, num_hosts: u32, factor: f64) -> (u32, u32, u32, u32) {
    // Collect valid configs
    let mut configs = Vec::new();
    for h in 1..k {
        for a in 2..k {
            let remaining = k - (a - 1) - h;
            if remaining <= 0 { break; }
            if remaining % lph != 0 { continue; }
            let p = remaining / lph;
            if p < 1 { continue; }
            let g_max = a * h + 1;
            for g in 2..=g_max {
                if (a * g * h) % 2 != 0 { continue; }
                let capacity = p * a * g;
                if capacity < num_hosts { continue; }
                configs.push((a, h, p, g));
            }
        }
    }
    if configs.is_empty() {
        panic!("No valid High-BW Dragonfly config for {} hosts", num_hosts);
    }

    let _min_routers = configs.iter().map(|&(_,_,_,g)| g).min().unwrap();  // wait, a*g
    let min_routers = configs.iter().map(|&(a,_,_,g)| a * g).min().unwrap();
    let cap = ((min_routers as f64 * factor).ceil()) as u32;

    let mut best = (0u32,0u32,0u32,0u32);
    let mut best_key = (u32::MAX, u32::MAX, u32::MAX, u32::MAX);

    for &(a, h, p, g) in &configs {
        let total = a * g;
        if total > cap { continue; }
        let key = (
            (a as i32 - 2 * h as i32).unsigned_abs(),
            (a as i32 - 2 * (p * lph) as i32).unsigned_abs(),
            total,
            p * a * g - num_hosts,
        );
        if key < best_key {
            best = (a, h, p, g);
            best_key = key;
        }
    }
    best
}

fn wire_global_links(a: u32, g: u32, h: u32) -> Vec<(u32, u32)> {
    // Same as dragonfly.rs for now (to avoid duplication, ideally factor out)
    let total_r = a * g;
    let target = a * h;
    let mut rem = vec![target; g as usize];
    let mut counts: std::collections::HashMap<(u32, u32), u32> = std::collections::HashMap::new();

    while rem.iter().sum::<u32>() > 0 {
        let g1 = rem.iter().enumerate().max_by_key(|(_, &v)| v).map(|(i,_)| i as u32).unwrap();
        if rem[g1 as usize] == 0 { break; }

        let candidates: Vec<u32> = (0..g).filter(|&gi| gi != g1 && rem[gi as usize] > 0).collect();
        if candidates.is_empty() { panic!("Unable to realize global links"); }

        let g2 = *candidates.iter().min_by_key(|&&gi| {
            let pk = if g1 < gi { (g1, gi) } else { (gi, g1) };
            let score = counts.get(&pk).copied().unwrap_or(0);
            (score, std::cmp::Reverse(rem[gi as usize]), gi)
        }).unwrap();

        let pk = if g1 < g2 { (g1, g2) } else { (g2, g1) };
        *counts.entry(pk).or_insert(0) += 1;
        rem[g1 as usize] -= 1;
        rem[g2 as usize] -= 1;
    }

    if rem.iter().any(|&v| v != 0) { panic!("Failed group degree"); }

    let mut r_ports = vec![0u32; total_r as usize];
    let mut links = Vec::new();

    let mut keys: Vec<_> = counts.keys().cloned().collect();
    keys.sort();

    for pk in keys {
        let mult = counts[&pk];
        let (g1, g2) = pk;
        for _ in 0..mult {
            let best_r1 = (0..a).min_by_key(|&r| r_ports[(g1*a + r) as usize]).unwrap();
            let best_r2 = (0..a).min_by_key(|&r| r_ports[(g2*a + r) as usize]).unwrap();
            let src = g1 * a + best_r1;
            let dst = g2 * a + best_r2;
            if r_ports[src as usize] >= h || r_ports[dst as usize] >= h { panic!("budget"); }
            links.push((src, dst));
            r_ports[src as usize] += 1;
            r_ports[dst as usize] += 1;
        }
    }

    if r_ports.iter().any(|&v| v != h) { panic!("h not satisfied"); }
    links
}