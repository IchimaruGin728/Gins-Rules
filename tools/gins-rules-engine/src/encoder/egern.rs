use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::models::RuleSet;

pub fn encode(name: &str, rules: &RuleSet, out_dir: &Path, _cat: &str) -> Result<()> {
    fs::create_dir_all(out_dir)?;

    let out = out_dir.join(format!("{}.yaml", name));
    let mut egern_rules: Vec<serde_json::Value> = Vec::new();

    for s in &rules.domain_suffix {
        egern_rules.push(serde_json::json!({"domain_suffix": {"match": s, "policy": "Proxy"}}));
    }
    for s in &rules.domain {
        egern_rules.push(serde_json::json!({"domain": {"match": s, "policy": "Proxy"}}));
    }
    for s in &rules.domain_keyword {
        egern_rules.push(serde_json::json!({"domain_keyword": {"match": s, "policy": "Proxy"}}));
    }
    for s in &rules.domain_regex {
        egern_rules.push(serde_json::json!({"domain_regex": {"match": s, "policy": "Proxy"}}));
    }
    for s in &rules.domain_wildcard {
        egern_rules.push(serde_json::json!({"domain_wildcard": {"match": s, "policy": "Proxy"}}));
    }
    for s in &rules.ip_cidr {
        let key = if s.contains(':') { "ip_cidr6" } else { "ip_cidr" };
        egern_rules.push(serde_json::json!({key: {"match": s, "policy": "Proxy", "no_resolve": "true"}}));
    }
    for s in &rules.ip_asn {
        let asn_val = if s.starts_with("AS") {
            s.to_string()
        } else {
            format!("AS{}", s)
        };
        egern_rules.push(serde_json::json!({"asn": {"match": asn_val, "policy": "Proxy"}}));
    }
    // Egern does not support process_name in rule-set
    for s in &rules.user_agent {
        egern_rules.push(serde_json::json!({"user_agent": {"match": s, "policy": "Proxy"}}));
    }

    if !egern_rules.is_empty() {
        let mut wrap = serde_json::Map::new();
        wrap.insert(
            "rules".to_string(),
            serde_json::Value::Array(egern_rules),
        );
        let yaml_str = serde_yaml::to_string(&wrap)?;
        fs::write(&out, format!("# Gins-Rules: {}\n{}", name, yaml_str))?;
    }
    Ok(())
}
