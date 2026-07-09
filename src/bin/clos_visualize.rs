use clap::Parser;
use std::path::PathBuf;
use std::fs;

use clos_dragonfly_simulation_rust::viz;

#[derive(Parser)]
#[command(name = "clos-visualize")]
struct Args {
    input: PathBuf,
    #[arg(long)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let data = fs::read_to_string(&args.input).expect("failed to read JSON");
    let links: Vec<[u32; 3]> = serde_json::from_str(&data).expect("invalid topology JSON");

    let out = args.output.unwrap_or_else(|| {
        let mut p = args.input.clone();
        p.set_extension("png");
        p
    });

    if let Err(e) = viz::visualize_clos(&links, &out, &format!("CLOS {}", args.input.display())) {
        eprintln!("Visualization error: {}", e);
        std::process::exit(1);
    }

    println!("Diagram written to: {}", out.display());
}
