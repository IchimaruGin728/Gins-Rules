use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::models::RuleSet;

pub fn encode(name: &str, rules: &RuleSet, out_dir: &Path, _cat: &str) -> Result<()> {
    fs::create_dir_all(out_dir)?;

    let out = out_dir.join(format!("{}.lsr", name));
    let mut p = Vec::new();

    for s in &rules.domain_suffix {
        p.push(format!("DOMAIN-SUFFIX,{}", s));
    }
    for s in &rules.domain {
        p.push(format!("DOMAIN,{}", s));
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
    for s in &rules.ip_asn {
        p.push(format!("IP-ASN,{}", s));
    }
    for s in &rules.user_agent {
        p.push(format!("USER-AGENT,{}", s));
    }
    // Loon does not support PROCESS-NAME in rule sets

    p.sort_unstable();
    if !p.is_empty() {
        fs::write(&out, p.join("\n") + "\n")?;
    }
    Ok(())
}
