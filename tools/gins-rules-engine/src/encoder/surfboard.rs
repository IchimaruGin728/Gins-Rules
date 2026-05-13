use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::models::RuleSet;

pub fn encode(name: &str, rules: &RuleSet, out_dir: &Path, _cat: &str) -> Result<()> {
    fs::create_dir_all(out_dir)?;

    let out = out_dir.join(format!("{}.list", name));
    let mut p = Vec::new();

    // Performance-tier order
    for s in &rules.domain {
        p.push(format!("DOMAIN,{}", s));
    }
    for s in &rules.domain_suffix {
        p.push(format!("DOMAIN-SUFFIX,{}", s));
    }
    for s in &rules.domain_keyword {
        p.push(format!("DOMAIN-KEYWORD,{}", s));
    }
    for s in &rules.ip_cidr {
        p.push(format!(
            "{},{}",
            if s.contains(':') { "IP-CIDR6" } else { "IP-CIDR" },
            s
        ));
    }
    for s in &rules.process_name {
        p.push(format!("PROCESS-NAME,{}", s));
    }
    // Surfboard does not support USER-AGENT, IP-ASN

    p.sort_unstable();
    if !p.is_empty() {
        fs::write(&out, p.join("\n") + "\n")?;
    }

    // .txt (domain-only optimized)
    let mut txt: Vec<String> = rules
        .domain_suffix
        .iter()
        .map(|s| format!(".{}", s))
        .collect();
    txt.extend(rules.domain.iter().map(|s| s.to_string()));
    txt.sort_unstable();
    if !txt.is_empty() {
        fs::write(out_dir.join(format!("{}.txt", name)), txt.join("\n") + "\n")?;
    }

    Ok(())
}
