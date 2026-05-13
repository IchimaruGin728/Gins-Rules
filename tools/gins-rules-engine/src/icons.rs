use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct IconSource {
    name: String,
    url: String,
    theme: String,
    enabled: bool,
}

#[derive(Debug, Deserialize)]
struct RawIcon {
    name: Option<String>,
    tag: Option<String>,
    url: Option<String>,
}

#[derive(Debug, Serialize)]
struct NormalizedIcon {
    name: String,
    url: String,
    source: String,
    theme: String,
}

pub async fn run(root: &str) -> Result<()> {
    let root = PathBuf::from(root).canonicalize()?;
    let config_path = root.join("source/icons.json");
    let data = tokio::fs::read(&config_path).await?;
    let sources: Vec<IconSource> = serde_json::from_slice(&data)?;

    println!("🖼 Synchronizing icon catalog from {} sources...", sources.len());

    let client = reqwest::Client::builder()
        .user_agent("Gins-Rules-Engine/2.0")
        .build()?;

    let mut all_icons: Vec<NormalizedIcon> = Vec::new();
    let mut join_set = tokio::task::JoinSet::new();

    for src in sources {
        if !src.enabled {
            continue;
        }
        let c = client.clone();
        join_set.spawn(async move {
            println!("  [Icon] Downloading {}...", src.name);
            let mut req = c.get(&src.url);
            if src.url.contains("kelee.one") {
                req = req.header("User-Agent", "Loon/338 CFNetwork/1498.700.2 Darwin/23.6.0");
            }
            let text = req.send().await?.text().await?;

            let raw: Vec<RawIcon> = serde_json::from_str(&text)?;
            let mut normalized = Vec::new();
            for r in raw {
                if let Some(url) = r.url {
                    if !url.is_empty() {
                        normalized.push(NormalizedIcon {
                            name: r.name.or(r.tag).unwrap_or_else(|| "icon".to_string()),
                            url,
                            source: src.name.clone(),
                            theme: src.theme.clone(),
                        });
                    }
                }
            }
            Ok::<_, anyhow::Error>(normalized)
        });
    }

    while let Some(result) = join_set.join_next().await {
        if let Ok(Ok(icons)) = result {
            all_icons.extend(icons);
        } else if let Ok(Err(e)) = result {
            eprintln!("  ❌ Error parsing icons: {}", e);
        }
    }

    all_icons.sort_by(|a, b| a.name.cmp(&b.name));

    let compiled_dir = root.join("compiled");
    tokio::fs::create_dir_all(&compiled_dir).await?;
    let compiled_path = compiled_dir.join("Gins-Icons.json");
    let json_bytes = serde_json::to_vec_pretty(&all_icons)?;
    tokio::fs::write(&compiled_path, &json_bytes).await?;

    let dashboard_path = root.join("dashboard/public/icons-catalog.json");
    if let Some(parent) = dashboard_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&dashboard_path, &json_bytes).await?;

    let hash_json = serde_json::json!({
        "sha256": json_bytes.len().to_string(),
        "total": all_icons.len().to_string()
    });
    tokio::fs::write(
        root.join("source/icons-hash.json"),
        serde_json::to_string(&hash_json)?,
    )
    .await?;

    println!("✨ Saved {} icons", all_icons.len());
    Ok(())
}
