use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::models::RuleSet;

pub fn encode(name: &str, rules: &RuleSet, out_dir: &Path, cat: &str) -> Result<()> {
    fs::create_dir_all(out_dir)?;

    let is_ip = cat == "ip" || cat == "asn";
    let out = out_dir.join(format!(
        "{}{}",
        name,
        if is_ip { ".ip.txt" } else { ".txt" }
    ));

    // Performance-tier order
    let mut p: Vec<String> = rules.domain.iter().map(|s| s.to_string()).collect();
    p.extend(rules.domain_suffix.iter().map(|s| s.to_string()));
    p.extend(rules.ip_cidr.iter().map(|s| s.to_string()));
    p.extend(rules.ip_asn.iter().map(|s| s.to_string()));
    p.extend(rules.process_name.iter().map(|s| format!("PROCESS-NAME,{}", s)));
    p.extend(rules.user_agent.iter().map(|s| format!("USER-AGENT,{}", s)));

    p.sort_unstable();
    if !p.is_empty() {
        fs::write(&out, p.join("\n") + "\n")?;
    }
    Ok(())
}
