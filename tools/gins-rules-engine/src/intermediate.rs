use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

use crate::models::RuleSet;

#[derive(Debug, Serialize)]
struct IntermediateOutput {
    version: u32,
    timestamp: String,
    categories: BTreeMap<String, BTreeMap<String, RuleSet>>,
}

pub fn write(
    categories: &BTreeMap<String, BTreeMap<String, RuleSet>>,
    out_dir: &Path,
) -> Result<()> {
    let output = IntermediateOutput {
        version: 1,
        timestamp: chrono::Utc::now().to_rfc3339(),
        categories: categories.clone(),
    };

    let path = out_dir.join("intermediate.json");
    let json = serde_json::to_string(&output)?;
    std::fs::write(&path, json)?;
    println!("  📄 Wrote intermediate.json");
    Ok(())
}
