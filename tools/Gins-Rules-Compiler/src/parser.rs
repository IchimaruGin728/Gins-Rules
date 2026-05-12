use anyhow::Result;
use ecow::EcoString;
use Gins_Rules_Core::RuleSet;
use memmap2::MmapOptions;
use std::fs::File;
use std::path::Path;

pub fn parse_file(path: &Path) -> Result<RuleSet> {
    let file = File::open(path)?;
    if file.metadata()?.len() == 0 {
        return Ok(RuleSet::new());
    }
    
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let content = std::str::from_utf8(&mmap).unwrap_or("");
    
    let mut rules = RuleSet::new();
    
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        
        let line = line.split('#').next().unwrap().trim();
        let line = line.split("//").next().unwrap().trim();
        
        parse_line(line, &mut rules);
    }
    
    Ok(rules)
}

fn parse_line(line: &str, rules: &mut RuleSet) {
    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
    
    if parts.len() == 1 {
        let val = parts[0];
        if let Some(stripped) = val.strip_prefix("full:") {
            rules.domain.insert(EcoString::from(stripped));
        } else if let Some(stripped) = val.strip_prefix("domain:") {
            rules.domain.insert(EcoString::from(stripped));
        } else if let Some(stripped) = val.strip_prefix("regex:") {
            rules.domain_regex.insert(EcoString::from(stripped));
        } else if let Some(stripped) = val.strip_prefix("keyword:") {
            rules.domain_keyword.insert(EcoString::from(stripped));
        } else if val.starts_with('+') || val.starts_with('.') {
            rules.domain_suffix.insert(EcoString::from(val.trim_start_matches('+').trim_start_matches('.')));
        } else if val.starts_with("AS") && val[2..].parse::<u32>().is_ok() {
            rules.ip_asn.insert(EcoString::from(val));
        } else if val.contains('/') {
            rules.ip_cidr.insert(EcoString::from(val));
        } else {
            if val.parse::<std::net::IpAddr>().is_ok() {
                let suffix = if val.contains(':') { "/128" } else { "/32" };
                rules.ip_cidr.insert(EcoString::from(format!("{}{}", val, suffix)));
            } else {
                rules.domain_suffix.insert(EcoString::from(val));
            }
        }
        return;
    }
    
    let t = parts[0].to_uppercase();
    let val = parts[1];
    
    match t.as_str() {
        "DOMAIN-SUFFIX" | "HOST-SUFFIX" | "DOMAIN" | "HOST" => { rules.domain_suffix.insert(EcoString::from(val)); },
        "DOMAIN-KEYWORD" | "HOST-KEYWORD" => { rules.domain_keyword.insert(EcoString::from(val)); },
        "DOMAIN-REGEX" | "HOST-REGEX" => { rules.domain_regex.insert(EcoString::from(val)); },
        "HOST-WILDCARD" => { rules.domain_regex.insert(EcoString::from(format!("^{}$", val.replace(".", "\\.").replace("*", ".*")))); },
        "IP-CIDR" | "IP-CIDR6" | "IP6-CIDR" => { rules.ip_cidr.insert(EcoString::from(val)); },
        "IP-ASN" => { rules.ip_asn.insert(EcoString::from(val)); },
        _ => {
            if val.contains('.') {
                 rules.domain_suffix.insert(EcoString::from(val));
            }
        }
    }
}
