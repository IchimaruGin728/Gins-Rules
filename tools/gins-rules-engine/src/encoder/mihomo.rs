use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::models::RuleSet;

pub fn encode(name: &str, rules: &RuleSet, out_dir: &Path, _cat: &str) -> Result<()> {
    fs::create_dir_all(out_dir)?;

    if rules.is_empty() {
        return Ok(());
    }

    let has_complex = rules.has_complex_types();

    if has_complex {
        // Split into domain MRS, ipcidr MRS, and classical YAML
        let mut domain_rules = RuleSet::new();
        domain_rules.domain = rules.domain.clone();
        domain_rules.domain_suffix = rules.domain_suffix.clone();

        if !domain_rules.is_empty() {
            // Write domain text file for Go to convert to MRS
            write_domain_text(name, &domain_rules, out_dir)?;
        }

        let mut ip_rules = RuleSet::new();
        ip_rules.ip_cidr = rules.ip_cidr.clone();
        ip_rules.ip_asn = rules.ip_asn.clone();
        if !ip_rules.is_empty() {
            // Write ipcidr text file for Go to convert to MRS
            write_ipcidr_text(&format!("{}-ip", name), &ip_rules, out_dir)?;
        }

        // Classical YAML with all rules
        write_classical_yaml(name, rules, out_dir)?;
    } else {
        // Simple domain-only rules: write domain text + classical YAML
        write_domain_text(name, rules, out_dir)?;
        write_classical_yaml(name, rules, out_dir)?;
    }

    Ok(())
}

fn write_domain_text(name: &str, rules: &RuleSet, out_dir: &Path) -> Result<()> {
    let out = out_dir.join(format!("{}.mrs", name));
    let mut payload: Vec<String> = rules
        .domain_suffix
        .iter()
        .map(|s| format!(".{}", s))
        .collect();
    payload.extend(rules.domain.iter().map(|s| s.to_string()));
    payload.sort_unstable();

    if !payload.is_empty() {
        fs::write(&out, payload.join("\n") + "\n")?;
    }
    Ok(())
}

fn write_ipcidr_text(name: &str, rules: &RuleSet, out_dir: &Path) -> Result<()> {
    let out = out_dir.join(format!("{}.mrs", name));
    let mut payload: Vec<String> = rules.ip_cidr.iter().map(|s| s.to_string()).collect();
    payload.sort_unstable();

    if !payload.is_empty() {
        fs::write(&out, payload.join("\n") + "\n")?;
    }
    Ok(())
}

fn write_classical_yaml(name: &str, rules: &RuleSet, out_dir: &Path) -> Result<()> {
    let out = out_dir.join(format!("{}.yaml", name));
    let mut payload: Vec<String> = Vec::new();

    // Performance-tier order
    for s in &rules.domain {
        payload.push(format!("DOMAIN,{}", s));
    }
    for s in &rules.domain_suffix {
        payload.push(format!("DOMAIN-SUFFIX,{}", s));
    }
    for s in &rules.domain_keyword {
        payload.push(format!("DOMAIN-KEYWORD,{}", s));
    }
    for s in &rules.domain_wildcard {
        payload.push(format!("DOMAIN-WILDCARD,{}", s));
    }
    for s in &rules.domain_regex {
        payload.push(format!("DOMAIN-REGEX,{}", s));
    }
    for s in &rules.ip_cidr {
        payload.push(format!(
            "{},{},no-resolve",
            if s.contains(':') { "IP-CIDR6" } else { "IP-CIDR" },
            s
        ));
    }
    for s in &rules.ip_asn {
        payload.push(format!("IP-ASN,{},no-resolve", s));
    }
    for s in &rules.process_name {
        payload.push(format!("PROCESS-NAME,{}", s));
    }
    // mihomo does not support USER-AGENT

    payload.sort_unstable();

    if !payload.is_empty() {
        let mut yaml_obj = std::collections::HashMap::new();
        yaml_obj.insert("payload", &payload);
        let yaml_str = serde_yaml::to_string(&yaml_obj)?;
        fs::write(&out, format!("# Gins-Rules: {}\n{}", name, yaml_str))?;
    }
    Ok(())
}
