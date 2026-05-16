use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::models::RuleSet;

// Anywhere RoutingRuleType (from Swift source):
//   0 = IPv4 CIDR, 1 = IPv6 CIDR, 2 = Domain Suffix, 3 = Domain Keyword
// Text format: `<type>, <value>` with optional `name = <name>` header.

pub fn encode(name: &str, rules: &RuleSet, out_dir: &Path, _cat: &str) -> Result<()> {
    fs::create_dir_all(out_dir)?;

    let out = out_dir.join(format!("{}.arrs", name));
    let mut lines: Vec<String> = Vec::new();

    lines.push(format!("name = {}", name));

    // 2 = Domain Suffix
    for s in &rules.domain_suffix {
        lines.push(format!("2, {}", s));
    }
    // 3 = Domain Keyword
    for s in &rules.domain_keyword {
        lines.push(format!("3, {}", s));
    }
    // 0 = IPv4 CIDR, 1 = IPv6 CIDR
    for s in &rules.ip_cidr {
        if s.contains(':') {
            lines.push(format!("1, {}", s));
        } else {
            lines.push(format!("0, {}", s));
        }
    }

    if !lines.is_empty() {
        fs::write(&out, lines.join("\n") + "\n")?;
    }
    Ok(())
}
