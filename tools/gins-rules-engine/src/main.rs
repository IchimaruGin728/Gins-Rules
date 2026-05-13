mod encoder;
mod icons;
mod intermediate;
mod models;
mod optimizer;
mod parser;
mod sync;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about = "Gins-Rules Engine — High-performance rule parser and format generator")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Download upstream rule sources
    Sync {
        #[arg(short, long, default_value = ".")]
        root: String,
    },
    /// Download icon catalog
    Icons {
        #[arg(short, long, default_value = ".")]
        root: String,
    },
    /// Parse rules and generate all text formats + intermediate.json
    Parse {
        #[arg(short, long, default_value = ".")]
        root: String,
        #[arg(short, long, default_value = "compiled")]
        output: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Sync { root } => {
            sync::run(&root).await?;
        }
        Commands::Icons { root } => {
            icons::run(&root).await?;
        }
        Commands::Parse { root, output } => {
            encoder::run(&root, &output)?;
        }
    }

    Ok(())
}
