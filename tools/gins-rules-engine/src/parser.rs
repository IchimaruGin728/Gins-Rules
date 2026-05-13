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

        // Skip HTTP error responses that leaked into upstream data
        if line.starts_with('<') || line.starts_with("<!DOCTYPE") || line.starts_with("<html")
            || line == "404: Not Found" || line == "503: Service Unavailable"
            || line == "403: Forbidden" {
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
        parse_single(parts[0], rules);
    } else {
        parse_prefixed(parts[0], parts[1], rules);
    }
}

/// Parse a single-value line (no comma separator).
/// Handles prefixed formats like `domain:`, `regex:`, bare domains, CIDRs, etc.
fn parse_single(val: &str, rules: &mut RuleSet) {
    if val.is_empty() {
        return;
    }

    // Prefixed formats: strip prefix and insert into the appropriate set
    if let Some((prefix, value)) = split_prefix(val) {
        match prefix {
            "full" | "domain" => { rules.domain.insert(CompactString::new(value)); }
            "regex" | "regexp" => { rules.domain_regex.insert(CompactString::new(value)); }
            "keyword" => { rules.domain_keyword.insert(CompactString::new(value)); }
            "process" => { rules.process_name.insert(CompactString::new(value)); }
            "user-agent" => { rules.user_agent.insert(CompactString::new(value)); }
            _ => {}
        }
        return;
    }

    // Bare values: classify by shape
    if val.starts_with('+') || val.starts_with('.') {
        rules.domain_suffix.insert(CompactString::new(
            val.trim_start_matches('+').trim_start_matches('.'),
        ));
    } else if val.starts_with("AS") && val[2..].parse::<u32>().is_ok() {
        rules.ip_asn.insert(CompactString::new(val));
    } else if val.contains('/') {
        rules.ip_cidr.insert(CompactString::new(val));
    } else if let Ok(addr) = val.parse::<std::net::IpAddr>() {
        let suffix = if addr.is_ipv6() { "/128" } else { "/32" };
        rules.ip_cidr.insert(CompactString::new(format!("{val}{suffix}")));
    } else {
        rules.domain_suffix.insert(CompactString::new(val));
    }
}

/// Parse a comma-separated rule line (e.g. `DOMAIN-SUFFIX,example.com,PROXY`).
fn parse_prefixed(rule_type: &str, value: &str, rules: &mut RuleSet) {
    let cs = || CompactString::new(value);

    match rule_type.to_uppercase().as_str() {
        "DOMAIN-SUFFIX" | "HOST-SUFFIX" => { rules.domain_suffix.insert(cs()); }
        "DOMAIN" | "HOST" => { rules.domain.insert(cs()); }
        "DOMAIN-KEYWORD" | "HOST-KEYWORD" => { rules.domain_keyword.insert(cs()); }
        "DOMAIN-REGEX" | "HOST-REGEX" => { rules.domain_regex.insert(cs()); }
        "DOMAIN-WILDCARD" | "HOST-WILDCARD" => { rules.domain_wildcard.insert(cs()); }
        "IP-CIDR" | "IP-CIDR6" | "IP6-CIDR" => { rules.ip_cidr.insert(cs()); }
        "IP-ASN" => { rules.ip_asn.insert(cs()); }
        "PROCESS-NAME" => { rules.process_name.insert(cs()); }
        "USER-AGENT" => { rules.user_agent.insert(cs()); }
        "GEOIP" => {
            // GEOIP not supported in intermediate model; should be pre-resolved to IP-CIDR
            eprintln!("WARN: Skipping GEOIP rule: {value}");
        }
        _ if value.contains('.') && !value.contains('/') => {
            // Fallback: looks like a domain
            rules.domain_suffix.insert(cs());
        }
        _ => {}
    }
}

/// Known single-value prefixes (without the `:`).
const KNOWN_PREFIXES: &[&str] = &[
    "full", "domain", "regex", "regexp", "keyword", "process", "user-agent",
];

/// Split a value on the first `:` to extract a prefix.
/// Only recognizes known prefixes to avoid false matches on colons in
/// IPv6 addresses, comments, etc.
fn split_prefix(val: &str) -> Option<(&str, &str)> {
    let idx = val.find(':')?;
    let (prefix, rest) = val.split_at(idx);
    if !KNOWN_PREFIXES.contains(&prefix) || rest.len() < 2 {
        return None;
    }
    Some((prefix, &rest[1..]))
}
