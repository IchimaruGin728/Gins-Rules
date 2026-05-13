use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::models::RuleSet;

// Anywhere format type mapping (v2ray GeoSite DomainType)
// 2 = domain_suffix, 3 = domain (exact), 4 = keyword, 5 = regex
// 6 = ip_cidr, 7 = ip_asn

pub fn encode(name: &str, rules: &RuleSet, out_dir: &Path, _cat: &str) -> Result<()> {
    fs::create_dir_all(out_dir)?;

    let out = out_dir.join(format!("{}.json", name));
    let mut p: Vec<serde_json::Value> = Vec::new();

    for s in &rules.domain_suffix {
        p.push(serde_json::json!({"type": 2, "value": s}));
    }
    for s in &rules.domain {
        p.push(serde_json::json!({"type": 3, "value": s}));
    }
    for s in &rules.domain_keyword {
        p.push(serde_json::json!({"type": 4, "value": s}));
    }
    for s in &rules.domain_regex {
        p.push(serde_json::json!({"type": 5, "value": s}));
    }
    for s in &rules.ip_cidr {
        p.push(serde_json::json!({"type": 6, "value": s}));
    }
    for s in &rules.ip_asn {
        p.push(serde_json::json!({"type": 7, "value": s}));
    }

    if !p.is_empty() {
        fs::write(&out, serde_json::to_string(&p)?)?;
    }
    Ok(())
}
