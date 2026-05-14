use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::models::RuleSet;

pub fn encode(name: &str, rules: &RuleSet, out_dir: &Path, _cat: &str) -> Result<()> {
    fs::create_dir_all(out_dir)?;

    let out = out_dir.join(format!("{}.yaml", name));
    let mut egern_sets: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();

    // Use rule-set format (_set suffix) as per egern docs
    if !rules.domain.is_empty() {
        egern_sets.insert(
            "domain_set".to_string(),
            serde_json::Value::Array(rules.domain.iter().map(|s| serde_json::Value::String(s.to_string())).collect()),
        );
    }
    if !rules.domain_suffix.is_empty() {
        egern_sets.insert(
            "domain_suffix_set".to_string(),
            serde_json::Value::Array(rules.domain_suffix.iter().map(|s| serde_json::Value::String(s.to_string())).collect()),
        );
    }
    if !rules.domain_keyword.is_empty() {
        egern_sets.insert(
            "domain_keyword_set".to_string(),
            serde_json::Value::Array(rules.domain_keyword.iter().map(|s| serde_json::Value::String(s.to_string())).collect()),
        );
    }
    if !rules.domain_regex.is_empty() {
        egern_sets.insert(
            "domain_regex_set".to_string(),
            serde_json::Value::Array(rules.domain_regex.iter().map(|s| serde_json::Value::String(s.to_string())).collect()),
        );
    }
    if !rules.domain_wildcard.is_empty() {
        egern_sets.insert(
            "domain_wildcard_set".to_string(),
            serde_json::Value::Array(rules.domain_wildcard.iter().map(|s| serde_json::Value::String(s.to_string())).collect()),
        );
    }

    // IP rules: split into ip_cidr_set and ip_cidr6_set
    let mut ipv4_list: Vec<serde_json::Value> = Vec::new();
    let mut ipv6_list: Vec<serde_json::Value> = Vec::new();
    for s in &rules.ip_cidr {
        if s.contains(':') {
            ipv6_list.push(serde_json::Value::String(s.to_string()));
        } else {
            ipv4_list.push(serde_json::Value::String(s.to_string()));
        }
    }
    if !ipv4_list.is_empty() {
        egern_sets.insert("ip_cidr_set".to_string(), serde_json::Value::Array(ipv4_list));
    }
    if !ipv6_list.is_empty() {
        egern_sets.insert("ip_cidr6_set".to_string(), serde_json::Value::Array(ipv6_list));
    }

    // ASN rules
    if !rules.ip_asn.is_empty() {
        let asn_list: Vec<serde_json::Value> = rules
            .ip_asn
            .iter()
            .map(|s| {
                let asn_val = if s.starts_with("AS") {
                    s.to_string()
                } else {
                    format!("AS{}", s)
                };
                serde_json::Value::String(asn_val)
            })
            .collect();
        egern_sets.insert("asn_set".to_string(), serde_json::Value::Array(asn_list));
    }

    // User-Agent rules
    if !rules.user_agent.is_empty() {
        egern_sets.insert(
            "user_agent_set".to_string(),
            serde_json::Value::Array(rules.user_agent.iter().map(|s| serde_json::Value::String(s.to_string())).collect()),
        );
    }

    // Add no_resolve for IP rules
    if !rules.ip_cidr.is_empty() || !rules.ip_asn.is_empty() {
        egern_sets.insert("no_resolve".to_string(), serde_json::Value::Bool(true));
    }

    if !egern_sets.is_empty() {
        let yaml_str = serde_yaml::to_string(&serde_json::Value::Object(egern_sets))?;
        fs::write(&out, format!("# Gins-Rules: {}\n{}", name, yaml_str))?;
    }
    Ok(())
}
