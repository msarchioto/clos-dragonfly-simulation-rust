use clap::Parser;
use std::path::PathBuf;

use clos_dragonfly_simulation_rust::dragonfly_high_bw as hbw;

#[derive(Parser)]
#[command(name = "dragonfly-high-bw-generate")]
struct Args {
    #[arg(long)]
    switch_throughput: u32,
    #[arg(long)]
    nic_throughput: u32,
    #[arg(long)]
    link_bandwidth: u32,
    #[arg(long)]
    num_hosts: u32,
    #[arg(long, default_value_t = 2.0)]
    router_budget_factor: f64,
    #[arg(long)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    match hbw::generate(
        args.switch_throughput,
        args.nic_throughput,
        args.link_bandwidth,
        args.num_hosts,
        args.router_budget_factor,
    ) {
        Ok(topo) => {
            let output_path = args.output.unwrap_or_else(|| {
                PathBuf::from(format!("output_dragonfly_high_bw/dragonfly_{}.json", args.num_hosts))
            });

            let _ = topo.write_json(&output_path);
            println!("{}", topo.summary());
            println!("\nTopology written to: {}", output_path.display());
            let png = output_path.with_extension("png");
            println!("For high-quality diagram (using Python matplotlib):");
            println!("  cd ../clos-dragonfly-simulation && uv run dragonfly-visualize {} --output {}", output_path.display(), png.display());
        }
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(1);
        }
    }
}