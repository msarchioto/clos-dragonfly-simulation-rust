use clap::Parser;
use std::path::PathBuf;

use clos_dragonfly_simulation_rust::clos;

#[derive(Parser, Debug)]
#[command(name = "clos-generate", about = "Generate a 2-layer CLOS topology")]
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

    #[arg(long)]
    version: bool,
}

fn main() {
    let args = Args::parse();

    if args.version {
        println!("clos-generate {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    match clos::generate(
        args.switch_throughput,
        args.nic_throughput,
        args.link_bandwidth,
        args.num_hosts,
    ) {
        Ok(topo) => {
            let output_path = args.output.unwrap_or_else(|| {
                PathBuf::from(format!("output_clos/topo_{}.json", args.num_hosts))
            });

            if let Err(e) = topo.write_json(&output_path) {
                eprintln!("ERROR writing JSON: {}", e);
                std::process::exit(1);
            }

            println!("{}", topo.summary());
            println!("\nTopology written to: {}", output_path.display());

            let png_path = output_path.with_extension("png");
            let _ = clos_dragonfly_simulation_rust::viz::visualize_clos(
                &topo.to_json(),
                &png_path,
                &format!("2-Layer CLOS Topology ({})", output_path.display()),
            );
            if png_path.exists() {
                println!("Diagram written to: {}", png_path.display());
            }
        }
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(1);
        }
    }
}
