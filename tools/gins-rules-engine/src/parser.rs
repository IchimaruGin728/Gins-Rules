use anyhow::Result;
use compact_str::CompactString;
use memmap2::MmapOptions;
use std::fs::File;
use std::path::Path;

use crate::models::RuleSet;

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

        // Skip YAML list prefix
        let line = line.strip_prefix("- ").unwrap_or(line);

        parse_line(line, &mut rules);
    }

    Ok(rules)
}

fn parse_line(line: &str, rules: &mut RuleSet) {
    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

    if parts.len() == 1 {
        let val = parts[0];
        if val.is_empty() {
            return;
        }
        if let Some(stripped) = val.strip_prefix("full:") {
            rules.domain.insert(CompactString::new(stripped));
        } else if let Some(stripped) = val.strip_prefix("domain:") {
            rules.domain.insert(CompactString::new(stripped));
        } else if let Some(stripped) = val.strip_prefix("regex:") {
            rules.domain_regex.insert(CompactString::new(stripped));
        } else if let Some(stripped) = val.strip_prefix("keyword:") {
            rules.domain_keyword.insert(CompactString::new(stripped));
        } else if val.starts_with('+') || val.starts_with('.') {
            rules
                .domain_suffix
                .insert(CompactString::new(val.trim_start_matches('+').trim_start_matches('.')));
        } else if val.starts_with("AS") && val[2..].parse::<u32>().is_ok() {
            rules.ip_asn.insert(CompactString::new(val));
        } else if val.contains('/') {
            rules.ip_cidr.insert(CompactString::new(val));
        } else if val.parse::<std::net::IpAddr>().is_ok() {
            let suffix = if val.contains(':') { "/128" } else { "/32" };
            rules
                .ip_cidr
                .insert(CompactString::new(format!("{}{}", val, suffix)));
        } else {
            rules.domain_suffix.insert(CompactString::new(val));
        }
        return;
    }

    let t = parts[0].to_uppercase();
    let val = parts[1];

    match t.as_str() {
        "DOMAIN-SUFFIX" | "HOST-SUFFIX" => {
            rules.domain_suffix.insert(CompactString::new(val));
        }
        "DOMAIN" | "HOST" => {
            rules.domain.insert(CompactString::new(val));
        }
        "DOMAIN-KEYWORD" | "HOST-KEYWORD" => {
            rules.domain_keyword.insert(CompactString::new(val));
        }
        "DOMAIN-REGEX" | "HOST-REGEX" => {
            rules.domain_regex.insert(CompactString::new(val));
        }
        "DOMAIN-WILDCARD" | "HOST-WILDCARD" => {
            rules.domain_wildcard.insert(CompactString::new(val));
        }
        "IP-CIDR" | "IP-CIDR6" | "IP6-CIDR" => {
            rules.ip_cidr.insert(CompactString::new(val));
        }
        "IP-ASN" => {
            rules.ip_asn.insert(CompactString::new(val));
        }
        "PROCESS-NAME" => {
            rules.process_name.insert(CompactString::new(val));
        }
        "USER-AGENT" => {
            rules.user_agent.insert(CompactString::new(val));
        }
        "GEOIP" => {
            // GEOIP rules are not supported in the intermediate model
            // They should be pre-resolved to IP-CIDR by upstream sources
            eprintln!("WARN: Skipping GEOIP rule: {}", val);
        }
        _ => {
            // Fallback: if it looks like a domain, treat as domain_suffix
            if val.contains('.') && !val.contains('/') {
                rules.domain_suffix.insert(CompactString::new(val));
            }
        }
    }
}
