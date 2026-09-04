//! CLI entry point for mdsite.

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "mdsite", about = "Minimal static site generator from Markdown")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a static site from Markdown files.
    Build {
        /// Input directory of `.md` files
        #[arg(long, value_name = "DIR")]
        input: PathBuf,
        /// Output directory for HTML (and copied Markdown)
        #[arg(long, value_name = "DIR")]
        output: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Commands::Build { input, output } => match mdsite::build(&input, &output) {
            Ok(()) => {
                eprintln!(
                    "Built site from {} into {}",
                    input.display(),
                    output.display()
                );
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
    }
}
