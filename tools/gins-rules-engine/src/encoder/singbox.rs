use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::models::RuleSet;

pub fn encode(name: &str, rules: &RuleSet, out_dir: &Path, _cat: &str) -> Result<()> {
    fs::create_dir_all(out_dir)?;

    // JSON format
    let out = out_dir.join(format!("{}.json", name));
    let mut obj = serde_json::Map::new();
    obj.insert("version".to_string(), serde_json::json!(1));

    let mut rule_obj = serde_json::Map::new();

    if !rules.domain_suffix.is_empty() {
        let mut v: Vec<_> = rules.domain_suffix.iter().map(|s| s.to_string()).collect();
        v.sort_unstable();
        rule_obj.insert("domain_suffix".to_string(), serde_json::json!(v));
    }
    if !rules.domain.is_empty() {
        let mut v: Vec<_> = rules.domain.iter().map(|s| s.to_string()).collect();
        v.sort_unstable();
        rule_obj.insert("domain".to_string(), serde_json::json!(v));
    }
    if !rules.domain_keyword.is_empty() {
        let mut v: Vec<_> = rules.domain_keyword.iter().map(|s| s.to_string()).collect();
        v.sort_unstable();
        rule_obj.insert("domain_keyword".to_string(), serde_json::json!(v));
    }
    if !rules.domain_regex.is_empty() {
        let mut v: Vec<_> = rules.domain_regex.iter().map(|s| s.to_string()).collect();
        v.sort_unstable();
        rule_obj.insert("domain_regex".to_string(), serde_json::json!(v));
    }
    if !rules.ip_cidr.is_empty() {
        let mut v: Vec<_> = rules.ip_cidr.iter().map(|s| s.to_string()).collect();
        v.sort_unstable();
        rule_obj.insert("ip_cidr".to_string(), serde_json::json!(v));
    }
    if !rules.ip_asn.is_empty() {
        let mut v: Vec<_> = rules
            .ip_asn
            .iter()
            .filter_map(|s| s.trim_start_matches("AS").parse::<u32>().ok())
            .collect();
        v.sort_unstable();
        rule_obj.insert("asn".to_string(), serde_json::json!(v));
    }
    if !rules.process_name.is_empty() {
        let mut v: Vec<_> = rules.process_name.iter().map(|s| s.to_string()).collect();
        v.sort_unstable();
        rule_obj.insert("process_name".to_string(), serde_json::json!(v));
    }
    // sing-box does not support user_agent in rule-sets

    if rule_obj.is_empty() {
        return Ok(());
    }

    obj.insert("rules".to_string(), serde_json::json!(vec![rule_obj]));
    fs::write(&out, serde_json::to_string(&obj)?)?;

    // SRS is handled by Go binary generator — we write the JSON source file
    // that Go will use as input for `sing-box rule-set compile`

    Ok(())
}
