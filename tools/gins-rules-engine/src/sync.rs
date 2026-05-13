use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct UpstreamSource {
    name: String,
    url: String,
    category: String,
    target: String,
    enabled: bool,
}

pub async fn run(root: &str) -> Result<()> {
    let root = PathBuf::from(root).canonicalize()?;
    let sources_path = root.join("source/sources.json");
    let data = tokio::fs::read(&sources_path).await?;
    let sources: Vec<UpstreamSource> = serde_json::from_slice(&data)?;

    let active: Vec<_> = sources.into_iter().filter(|s| s.enabled).collect();
    println!("📡 Syncing {} active rule sources...", active.len());

    let client = reqwest::Client::builder()
        .user_agent("Gins-Rules-Engine/2.0")
        .build()?;

    let mut collected: HashMap<String, HashMap<String, String>> = HashMap::new();

    let mut join_set = tokio::task::JoinSet::new();
    for src in active {
        let c = client.clone();
        join_set.spawn(async move {
            println!("  [Rule] Downloading {}...", src.name);
            let resp = c.get(&src.url).send().await?.text().await?;
            Ok::<_, anyhow::Error>((src.category, src.target, resp))
        });
    }

    while let Some(result) = join_set.join_next().await {
        if let Ok(Ok((cat, target, text))) = result {
            let cat_map = collected.entry(cat).or_default();
            let entry = cat_map.entry(target).or_default();
            entry.push_str(&text);
            entry.push('\n');
        } else if let Ok(Err(e)) = result {
            eprintln!("  ❌ Error downloading: {}", e);
        }
    }

    for (cat, targets) in collected {
        let dir = root.join("source/upstream").join(&cat);
        tokio::fs::create_dir_all(&dir).await?;
        for (target, content) in targets {
            let file_path = dir.join(format!("{}.txt", target));
            tokio::fs::write(&file_path, content).await?;
            println!("  ✅ Saved {}/{}", cat, target);
        }
    }

    println!("✨ Rule sync completed.");
    Ok(())
}
