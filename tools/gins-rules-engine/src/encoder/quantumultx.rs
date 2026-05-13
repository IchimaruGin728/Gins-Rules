use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::models::RuleSet;

pub fn encode(name: &str, rules: &RuleSet, out_dir: &Path, _cat: &str) -> Result<()> {
    fs::create_dir_all(out_dir)?;

    let out = out_dir.join(format!("{}.list", name));
    let mut p = Vec::new();

    for s in &rules.domain_suffix {
        p.push(format!("HOST-SUFFIX,{},PROXY", s));
    }
    for s in &rules.domain {
        p.push(format!("HOST,{},PROXY", s));
    }
    for s in &rules.domain_keyword {
        p.push(format!("HOST-KEYWORD,{},PROXY", s));
    }
    for s in &rules.domain_wildcard {
        p.push(format!("HOST-WILDCARD,{},PROXY", s));
    }
    for s in &rules.ip_cidr {
        p.push(format!(
            "{},{},PROXY",
            if s.contains(':') { "IP6-CIDR" } else { "IP-CIDR" },
            s
        ));
    }
    for s in &rules.ip_asn {
        p.push(format!("IP-ASN,{},PROXY", s));
    }
    for s in &rules.user_agent {
        p.push(format!("USER-AGENT,{},PROXY", s));
    }
    // QuanX does not support PROCESS-NAME

    p.sort_unstable();
    if !p.is_empty() {
        fs::write(&out, p.join("\n") + "\n")?;
    }
    Ok(())
}
