use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::models::RuleSet;

pub fn encode(name: &str, rules: &RuleSet, out_dir: &Path, _cat: &str) -> Result<()> {
    fs::create_dir_all(out_dir)?;

    let out = out_dir.join(format!("{}.list", name));
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
    for s in &rules.domain_regex {
        p.push(format!("DOMAIN-REGEX,{}", s));
    }
    for s in &rules.ip_cidr {
        p.push(format!(
            "{},{},no-resolve",
            if s.contains(':') { "IP-CIDR6" } else { "IP-CIDR" },
            s
        ));
    }
    for s in &rules.ip_asn {
        p.push(format!("IP-ASN,{},no-resolve", s));
    }
    for s in &rules.process_name {
        p.push(format!("PROCESS-NAME,{}", s));
    }
    for s in &rules.user_agent {
        p.push(format!("USER-AGENT,{}", s));
    }

    p.sort_unstable();
    if !p.is_empty() {
        fs::write(&out, p.join("\n") + "\n")?;
    }

    // Domainset (.domainset)
    let mut ds: Vec<String> = rules
        .domain_suffix
        .iter()
        .map(|s| format!(".{}", s))
        .collect();
    ds.extend(rules.domain.iter().map(|s| s.to_string()));
    ds.sort_unstable();
    if !ds.is_empty() {
        fs::write(out_dir.join(format!("{}.domainset", name)), ds.join("\n") + "\n")?;
    }

    Ok(())
}

pub fn encode_stash(name: &str, rules: &RuleSet, out_dir: &Path, cat: &str) -> Result<()> {
    // Stash uses the same format as mihomo
    super::mihomo::encode(name, rules, out_dir, cat)
}

pub fn encode_shadowrocket(
    name: &str,
    rules: &RuleSet,
    out_dir: &Path,
    _cat: &str,
) -> Result<()> {
    fs::create_dir_all(out_dir)?;

    let out = out_dir.join(format!("{}.list", name));
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
    // Shadowrocket does not support PROCESS-NAME, USER-AGENT, IP-ASN in rule sets

    p.sort_unstable();
    if !p.is_empty() {
        fs::write(&out, p.join("\n") + "\n")?;
    }

    // .txt (domain-only)
    let mut txt: Vec<String> = rules.domain_suffix.iter().map(|s| s.to_string()).collect();
    txt.extend(rules.domain.iter().map(|s| s.to_string()));
    txt.sort_unstable();
    if !txt.is_empty() {
        fs::write(out_dir.join(format!("{}.txt", name)), txt.join("\n") + "\n")?;
    }

    Ok(())
}
