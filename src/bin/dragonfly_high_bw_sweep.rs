use clap::Parser;
use std::fs;
use std::path::PathBuf;

use clos_dragonfly_simulation_rust::dragonfly_high_bw as hbw;

#[derive(Parser)]
#[command(name = "dragonfly-high-bw-sweep")]
struct Args {
    #[arg(long)]
    switch_throughput: u32,
    #[arg(long)]
    nic_throughput: u32,
    #[arg(long)]
    link_bandwidth: u32,
    #[arg(long, default_value = "output_dragonfly_high_bw")]
    output_dir: PathBuf,
    #[arg(long, default_value_t = 2.0)]
    router_budget_factor: f64,
    #[arg(long)]
    force: bool,
}

fn main() {
    let args = Args::parse();
    let hosts = vec![4u32, 8, 16, 32, 64, 128];
    let mut generated = vec![];
    let mut skipped = vec![];
    let mut failed = vec![];

    for &n in &hosts {
        let out = args.output_dir.join(format!("dragonfly_{}.json", n));
        if out.exists() && !args.force {
            skipped.push(n);
            continue;
        }
        match hbw::generate(
            args.switch_throughput,
            args.nic_throughput,
            args.link_bandwidth,
            n,
            args.router_budget_factor,
        ) {
            Ok(topo) => {
                let _ = fs::create_dir_all(&args.output_dir);
                let _ = topo.write_json(&out);
                let png = out.with_extension("png");
                if let Err(e) = clos_dragonfly_simulation_rust::viz::visualize_dragonfly(
                    &topo.to_json(),
                    &png,
                    &format!("Dragonfly High-BW Topology ({})", out.display()),
                    topo.num_hosts,
                    topo.routers_per_group,
                    topo.num_groups,
                ) {
                    println!("[OK] hosts={} -> {} (diagram warning: {})", n, out.display(), e);
                } else {
                    println!("[OK] hosts={} -> {} + {}", n, out.display(), png.display());
                }
                println!("{}", topo.summary());
                println!();
                generated.push(n);
            }
            Err(e) => {
                eprintln!("[FAIL] hosts={}: {}", n, e);
                failed.push(n);
            }
        }
    }
    println!("--- Sweep Summary ---");
    println!(
        "Generated: {}",
        if generated.is_empty() {
            "none".to_string()
        } else {
            format!("{:?}", generated)
        }
    );
    println!(
        "Skipped:   {}",
        if skipped.is_empty() {
            "none".to_string()
        } else {
            format!("{:?}", skipped)
        }
    );
    println!(
        "Failed:    {}",
        if failed.is_empty() {
            "none".to_string()
        } else {
            format!("{:?}", failed)
        }
    );
}
