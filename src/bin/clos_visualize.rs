use clap::Parser;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(name = "clos-visualize")]
struct Args {
    input: PathBuf,
    #[arg(long)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let out = args.output.unwrap_or_else(|| {
        let mut p = args.input.clone();
        p.set_extension("png");
        p
    });

    // Reverted to Python for best quality (identical to original matplotlib)
    // Assumes sibling Python project with uv
    let python_proj = "../clos-dragonfly-simulation";
    let status = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "cd {} && uv run clos-visualize {} --output {}",
            python_proj,
            args.input.display(),
            out.display()
        ))
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("Diagram written to: {}", out.display());
        }
        _ => {
            eprintln!(
                "Failed to run Python visualize. Make sure uv and the sibling Python project exist.\n\
                 Alternatively, run manually: cd {} && uv run clos-visualize {} --output {}",
                python_proj, args.input.display(), out.display()
            );
            std::process::exit(1);
        }
    }
}