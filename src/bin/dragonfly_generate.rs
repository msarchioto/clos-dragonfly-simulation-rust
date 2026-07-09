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
                PathBuf::from(format!(
                    "output_dragonfly/dragonfly_{}.json",
                    args.num_hosts
                ))
            });

            if let Err(e) = topo.write_json(&output_path) {
                eprintln!("ERROR: {}", e);
                std::process::exit(1);
            }

            println!("{}", topo.summary());
            println!("\nTopology written to: {}", output_path.display());
            let png = output_path.with_extension("png");
            let _ = clos_dragonfly_simulation_rust::viz::visualize_dragonfly(
                &topo.to_json(),
                &png,
                &format!("Dragonfly Topology ({})", output_path.display()),
                topo.num_hosts,
                topo.routers_per_group,
                topo.num_groups,
            );
            if png.exists() {
                println!("Diagram written to: {}", png.display());
            }
        }
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(1);
        }
    }
}
