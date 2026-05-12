mod config;
mod fetcher;

use anyhow::Result;
use clap::Parser;
use reqwest::Client;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = ".")]
    root: String,

    #[arg(help = "Command: sync, icons, he")]
    action: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let root = PathBuf::from(&args.root).canonicalize()?;
    let action = args.action.as_str();

    println!("🚀 [Gins-Rules Engine] Starting {}...", action);
    println!("📁 Base Directory: {}", root.display());

    let dirs = vec!["compiled", "source/upstream", "source/upstream/ip"];
    for dir in dirs {
        let path = root.join(dir);
        if !path.exists() {
            tokio::fs::create_dir_all(&path).await?;
            println!("  ✅ Created directory: {}", dir);
        }
    }

    let client = Client::builder()
        .user_agent("Loon/338 CFNetwork/1498.700.2 Darwin/23.6.0")
        .build()?;

    match action {
        "sync" => {
            let sources_path = root.join("source/sources.json");
            let data = tokio::fs::read(&sources_path).await?;
            let sources: Vec<config::UpstreamSource> = serde_json::from_slice(&data)?;
            println!("📡 Syncing {} active rule sources...", sources.len());
            fetcher::sync_rules(&client, sources, &root).await?;
            println!("✨ Rule sync completed.");
        }
        "icons" => {
            let config_path = root.join("source/icons.json");
            let data = tokio::fs::read(&config_path).await?;
            let sources: Vec<config::IconSource> = serde_json::from_slice(&data)?;
            println!("🖼 Synchronizing icon catalog...");
            fetcher::sync_icons(&client, sources, &root).await?;
            println!("✨ Icon catalog updated.");
        }
        "he" => {
            println!("🌐 Syncing Hurricane Electric prefixes... (Mock)");
        }
        _ => {
            eprintln!("⚠️ Unknown action: {}", action);
            std::process::exit(1);
        }
    }

    Ok(())
}
