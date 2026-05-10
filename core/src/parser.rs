use crate::models::Rules;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn parse_source(path: &Path) -> Result<Rules> {
    let mut r = Rules::default();
    let content = fs::read_to_string(path)?;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with("full:") {
            r.domain.push(line[5..].to_string());
        } else if line.starts_with("keyword:") {
            r.domain_keyword.push(line[8..].to_string());
        } else if line.starts_with("regexp:") {
            r.domain_regex.push(line[7..].to_string());
        } else if line.starts_with("process:") {
            r.process_name.push(line[8..].to_string());
        } else if line.starts_with("user-agent:") {
            r.user_agent.push(line[11..].to_string());
        } else if line.starts_with("asn:") {
            r.ip_asn.push(line[4..].to_string());
        } else if line.contains('/') {
            r.ip_cidr.push(line.to_string());
        } else {
            let domain = line.trim_start_matches("+.").trim_start_matches('.');
            r.domain_suffix.push(domain.to_string());
        }
    }
    Ok(r)
}

pub fn parse_content(content: &str) -> Rules {
    let mut r = Rules::default();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with("full:") {
            r.domain.push(line[5..].to_string());
        } else if line.starts_with("keyword:") {
            r.domain_keyword.push(line[8..].to_string());
        } else if line.starts_with("regexp:") {
            r.domain_regex.push(line[7..].to_string());
        } else if line.starts_with("process:") {
            r.process_name.push(line[8..].to_string());
        } else if line.starts_with("user-agent:") {
            r.user_agent.push(line[11..].to_string());
        } else if line.starts_with("asn:") {
            r.ip_asn.push(line[4..].to_string());
        } else if line.contains('/') {
            r.ip_cidr.push(line.to_string());
        } else {
            let domain = line.trim_start_matches("+.").trim_start_matches('.');
            r.domain_suffix.push(domain.to_string());
        }
    }
    r
}

pub fn merge_rules(mut a: Rules, b: Rules) -> Rules {
    a.domain_suffix.extend(b.domain_suffix);
    a.domain.extend(b.domain);
    a.domain_keyword.extend(b.domain_keyword);
    a.domain_regex.extend(b.domain_regex);
    a.ip_cidr.extend(b.ip_cidr);
    a.ip_asn.extend(b.ip_asn);
    a.process_name.extend(b.process_name);
    a.user_agent.extend(b.user_agent);
    a
}

pub fn unique(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v.dedup();
    v
}

pub fn sanitize_rules(mut rules: Rules) -> Rules {
    let force_proxy = vec![
        "tiktok.com",
        "tiktokv.com",
        "tiktokcdn.com",
        "byteoversea.com",
        "ibyteimg.com",
        "ibytedtos.com",
        "ipstatp.com",
        "muscdn.com",
        "musical.ly",
        "tik-tokapi.com",
    ];
    let is_force = |d: &str| {
        if d.contains("tiktok") || d.contains("tik-tok") {
            return true;
        }
        for p in &force_proxy {
            if d == *p || d.ends_with(&format!(".{}", p)) {
                return true;
            }
        }
        false
    };
    rules.domain.retain(|d| !is_force(d));
    rules.domain_suffix.retain(|d| !is_force(d));
    rules
}
