use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::models::RuleSet;

pub fn encode(name: &str, rules: &RuleSet, out_dir: &Path, _cat: &str) -> Result<()> {
    fs::create_dir_all(out_dir)?;

    let out = out_dir.join(format!("{}.list", name));
    let mut p: Vec<String> = rules
        .domain_suffix
        .iter()
        .map(|s| format!("+.{}", s))
        .collect();
    p.extend(rules.domain.iter().map(|s| s.to_string()));
    p.extend(rules.ip_cidr.iter().map(|s| s.to_string()));

    p.sort_unstable();
    if !p.is_empty() {
        fs::write(&out, p.join("\n") + "\n")?;
    }
    Ok(())
}
