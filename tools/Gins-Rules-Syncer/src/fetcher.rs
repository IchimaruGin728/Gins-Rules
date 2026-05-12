use crate::config::{IconSource, NormalizedIcon, RawIcon, UpstreamSource};
use anyhow::Result;
use futures::future::join_all;
use reqwest::Client;
use std::collections::HashMap;

pub async fn sync_rules(
    client: &Client,
    sources: Vec<UpstreamSource>,
    root: &std::path::Path,
) -> Result<()> {
    let mut tasks = vec![];
    
    for src in sources {
        if !src.enabled {
            continue;
        }
        let c = client.clone();
        tasks.push(tokio::spawn(async move {
            println!("  [Rule] Downloading {}...", src.name);
            let resp = c.get(&src.url).send().await?.text().await?;
            Ok::<_, anyhow::Error>((src.category, src.target, resp))
        }));
    }

    let results = join_all(tasks).await;
    
    let mut collected: HashMap<String, HashMap<String, String>> = HashMap::new();
    for res in results {
        if let Ok(Ok((cat, target, text))) = res {
            let cat_map = collected.entry(cat).or_default();
            let target_str = cat_map.entry(target).or_default();
            target_str.push_str(&text);
            target_str.push('\n');
        } else if let Ok(Err(e)) = res {
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
    
    Ok(())
}

pub async fn sync_icons(
    client: &Client,
    sources: Vec<IconSource>,
    root: &std::path::Path,
) -> Result<()> {
    let mut tasks = vec![];

    for src in sources {
        if !src.enabled {
            continue;
        }
        let c = client.clone();
        tasks.push(tokio::spawn(async move {
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
        }));
    }

    let results = join_all(tasks).await;
    let mut all_icons = Vec::new();
    for res in results {
        if let Ok(Ok(icons)) = res {
            all_icons.extend(icons);
        } else if let Ok(Err(e)) = res {
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

    println!("  ✅ Saved {} icons", all_icons.len());

    Ok(())
}
