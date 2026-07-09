use clap::Parser;
use std::path::PathBuf;

use clos_dragonfly_simulation_rust::dragonfly;

#[derive(Parser, Debug)]
#[command(name = "dragonfly-generate")]
struct Args {
    #[arg(long)]
    switch_throughput: u32,
    #[arg(long)]
    nic_throughput: u32,
    #[arg(long)]
    link_bandwidth: u32,
    #[arg(long)]
    num_hosts: u32,
    #[arg(long)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    match dragonfly::generate(
        args.switch_throughput,
        args.nic_throughput,
        args.link_bandwidth,
        args.num_hosts,
    ) {
        Ok(topo) => {
            let output_path = args.output.unwrap_or_else(|| {
                PathBuf::from(format!("output_dragonfly/dragonfly_{}.json", args.num_hosts))
            });

            if let Err(e) = topo.write_json(&output_path) {
                eprintln!("ERROR: {}", e);
                std::process::exit(1);
            }

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