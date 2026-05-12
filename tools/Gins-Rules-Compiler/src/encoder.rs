use crate::{mrs, srs};
use anyhow::Result;
use ecow::EcoString;
use Gins_Rules_Core::{Format, RuleSet};
use std::fs;
use std::path::Path;

pub fn compile(
    format: Format,
    name: &str,
    rules: &RuleSet,
    out_dir: &Path,
    is_ip: bool,
) -> Result<()> {
    fs::create_dir_all(out_dir)?;

    match format {
        Format::SingBox => to_singbox_json(name, rules, out_dir)?,
        Format::Srs => to_singbox_srs(name, rules, out_dir)?,
        Format::Mihomo | Format::Mrs | Format::Stash => {
            if is_ip {
                let behavior = if rules.ip_asn.is_empty() { "ipcidr" } else { "classical" };
                generate_mihomo_split(name, rules, out_dir, behavior)?;
            } else {
                let has_complex = !rules.domain_keyword.is_empty() 
                    || !rules.domain_regex.is_empty() 
                    || !rules.ip_asn.is_empty() 
                    || !rules.ip_cidr.is_empty();

                if has_complex {
                    let mut domain_rules = RuleSet::new();
                    domain_rules.domain = rules.domain.clone();
                    domain_rules.domain_suffix = rules.domain_suffix.clone();
                    
                    if !domain_rules.is_empty() {
                        generate_mihomo_split(name, &domain_rules, out_dir, "domain")?;
                    }

                    let mut ip_rules = RuleSet::new();
                    ip_rules.ip_cidr = rules.ip_cidr.clone();
                    ip_rules.ip_asn = rules.ip_asn.clone();
                    if !ip_rules.is_empty() {
                        let behavior = if ip_rules.ip_asn.is_empty() { "ipcidr" } else { "classical" };
                        generate_mihomo_split(&format!("{}-ip", name), &ip_rules, out_dir, behavior)?;
                    }

                    let mut kw_rules = RuleSet::new();
                    kw_rules.domain_keyword = rules.domain_keyword.clone();
                    kw_rules.domain_regex = rules.domain_regex.clone();
                    if !kw_rules.is_empty() {
                        generate_mihomo_split(&format!("{}-keyword", name), &kw_rules, out_dir, "classical")?;
                    }
                    
                    generate_mihomo_split(name, rules, out_dir, "classical")?;
                } else {
                    generate_mihomo_split(name, rules, out_dir, "domain")?;
                    generate_mihomo_split(name, rules, out_dir, "classical")?;
                }
            }
        }
        Format::Surge => to_surge(name, rules, out_dir, "domainset")?,
        Format::Shadowrocket => to_surge(name, rules, out_dir, "txt")?,
        Format::Loon => to_loon(name, rules, out_dir)?,
        Format::QuantumultX => to_quanx(name, rules, out_dir)?,
        Format::Surfboard => to_surfboard(name, rules, out_dir)?,
        Format::Exclave => to_exclave(name, rules, out_dir)?,
        Format::Anywhere => to_anywhere(name, rules, out_dir)?,
        Format::Egern => to_egern(name, rules, out_dir)?,
        Format::Text => to_text(name, rules, out_dir, is_ip)?,
    }
    Ok(())
}

fn to_singbox_json(name: &str, rules: &RuleSet, out_dir: &Path) -> Result<()> {
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
        let mut v: Vec<_> = rules.ip_asn.iter()
            .filter_map(|s| s.trim_start_matches("AS").parse::<u32>().ok())
            .collect();
        v.sort_unstable();
        rule_obj.insert("asn".to_string(), serde_json::json!(v));
    }
    
    obj.insert("rules".to_string(), serde_json::json!(vec![rule_obj]));
    fs::write(out, serde_json::to_string(&obj)?)?;
    Ok(())
}

fn to_singbox_srs(name: &str, rules: &RuleSet, out_dir: &Path) -> Result<()> {
    let out = out_dir.join(format!("{}.srs", name));
    let mut dom = rules.domain.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    dom.sort_unstable();
    let mut suf = rules.domain_suffix.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    suf.sort_unstable();
    let mut kw = rules.domain_keyword.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    kw.sort_unstable();
    let mut rx = rules.domain_regex.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    rx.sort_unstable();
    let mut cidr = rules.ip_cidr.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    cidr.sort_unstable();
    let mut asn = rules.ip_asn.iter().filter_map(|s| s.trim_start_matches("AS").parse::<u32>().ok()).collect::<Vec<_>>();
    asn.sort_unstable();
    
    srs::encode_srs(dom, suf, kw, rx, cidr, asn, &out)?;
    Ok(())
}

fn generate_mihomo_split(name: &str, rules: &RuleSet, out_dir: &Path, behavior: &str) -> Result<()> {
    let mut payload = Vec::new();
    
    if behavior == "classical" {
        for s in &rules.domain_suffix { payload.push(format!("DOMAIN-SUFFIX,{}", s)); }
        for s in &rules.domain { payload.push(format!("DOMAIN,{}", s)); }
        for s in &rules.domain_keyword { payload.push(format!("DOMAIN-KEYWORD,{}", s)); }
        for s in &rules.domain_regex { payload.push(format!("DOMAIN-REGEX,{}", s)); }
        for s in &rules.ip_cidr { payload.push(format!("{},{}", if s.contains(':') { "IP-CIDR6" } else { "IP-CIDR" }, s)); }
        for s in &rules.ip_asn { payload.push(format!("IP-ASN,{}", s)); }
        payload.sort_unstable();
    } else if behavior == "ipcidr" {
        payload = rules.ip_cidr.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        payload.sort_unstable();
    } else {
        payload = rules.domain_suffix.iter().map(|s| format!(".{}", s)).collect::<Vec<_>>();
        payload.extend(rules.domain.iter().map(|s| s.to_string()));
        payload.sort_unstable();
    }
    
    if payload.is_empty() { return Ok(()); }

    if behavior == "classical" {
        let out = out_dir.join(format!("{}.yaml", name));
        let mut yaml_obj = std::collections::HashMap::new();
        yaml_obj.insert("payload", &payload);
        let yaml_str = serde_yaml::to_string(&yaml_obj)?;
        fs::write(out, format!("# Gins-Rules: {}\n{}", name, yaml_str))?;
    } else {
        let out = out_dir.join(format!("{}.mrs", name));
        let refs: Vec<&str> = payload.iter().map(|s| s.as_str()).collect();
        mrs::encode_mrs(refs, &out)?;
    }
    Ok(())
}

fn to_surge(name: &str, rules: &RuleSet, out_dir: &Path, ext: &str) -> Result<()> {
    let out = out_dir.join(format!("{}.list", name));
    let mut p = Vec::new();
    for s in &rules.domain_suffix { p.push(format!("DOMAIN-SUFFIX,{}", s)); }
    for s in &rules.domain { p.push(format!("DOMAIN,{}", s)); }
    for s in &rules.domain_keyword { p.push(format!("DOMAIN-KEYWORD,{}", s)); }
    for s in &rules.ip_cidr { p.push(format!("{},{},no-resolve", if s.contains(':') { "IP-CIDR6" } else { "IP-CIDR" }, s)); }
    for s in &rules.ip_asn { p.push(format!("IP-ASN,{},no-resolve", s)); }
    p.sort_unstable();
    if !p.is_empty() {
        fs::write(&out, p.join("\n") + "\n")?;
    }
    
    if ext == "domainset" {
        let mut p = rules.domain_suffix.iter().map(|s| format!(".{}", s)).collect::<Vec<_>>();
        p.extend(rules.domain.iter().map(|s| s.to_string()));
        p.sort_unstable();
        if !p.is_empty() {
            fs::write(out_dir.join(format!("{}.{}", name, ext)), p.join("\n") + "\n")?;
        }
    } else if ext == "txt" {
        let mut p = rules.domain_suffix.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        p.extend(rules.domain.iter().map(|s| s.to_string()));
        p.sort_unstable();
        if !p.is_empty() {
            fs::write(out_dir.join(format!("{}.{}", name, ext)), p.join("\n") + "\n")?;
        }
    }
    Ok(())
}

fn to_loon(name: &str, rules: &RuleSet, out_dir: &Path) -> Result<()> {
    let out = out_dir.join(format!("{}.lsr", name));
    let mut p = Vec::new();
    for s in &rules.domain_suffix { p.push(format!("DOMAIN-SUFFIX,{}", s)); }
    for s in &rules.domain { p.push(format!("DOMAIN,{}", s)); }
    for s in &rules.domain_keyword { p.push(format!("DOMAIN-KEYWORD,{}", s)); }
    for s in &rules.ip_cidr { p.push(format!("{},{}", if s.contains(':') { "IP-CIDR6" } else { "IP-CIDR" }, s)); }
    for s in &rules.ip_asn { p.push(format!("IP-ASN,{}", s)); }
    p.sort_unstable();
    if !p.is_empty() {
        fs::write(&out, p.join("\n") + "\n")?;
    }
    Ok(())
}

fn to_quanx(name: &str, rules: &RuleSet, out_dir: &Path) -> Result<()> {
    let out = out_dir.join(format!("{}.list", name));
    let mut p = Vec::new();
    for s in &rules.domain_suffix { p.push(format!("HOST-SUFFIX,{},PROXY", s)); }
    for s in &rules.domain { p.push(format!("HOST,{},PROXY", s)); }
    for s in &rules.domain_keyword { p.push(format!("HOST-KEYWORD,{},PROXY", s)); }
    for s in &rules.ip_cidr { p.push(format!("{},{},PROXY", if s.contains(':') { "IP6-CIDR" } else { "IP-CIDR" }, s)); }
    for s in &rules.ip_asn { p.push(format!("IP-ASN,{},PROXY", s)); }
    p.sort_unstable();
    if !p.is_empty() {
        fs::write(&out, p.join("\n") + "\n")?;
    }
    Ok(())
}

fn to_surfboard(name: &str, rules: &RuleSet, out_dir: &Path) -> Result<()> {
    let out = out_dir.join(format!("{}.list", name));
    let mut p = Vec::new();
    for s in &rules.domain_suffix { p.push(format!("DOMAIN-SUFFIX,{}", s)); }
    for s in &rules.domain { p.push(format!("DOMAIN,{}", s)); }
    for s in &rules.domain_keyword { p.push(format!("DOMAIN-KEYWORD,{}", s)); }
    for s in &rules.ip_cidr { p.push(format!("{},{}", if s.contains(':') { "IP-CIDR6" } else { "IP-CIDR" }, s)); }
    for s in &rules.ip_asn { p.push(format!("IP-ASN,{}", s)); }
    p.sort_unstable();
    if !p.is_empty() {
        fs::write(&out, p.join("\n") + "\n")?;
    }
    
    let mut p2 = rules.domain_suffix.iter().map(|s| format!(".{}", s)).collect::<Vec<_>>();
    p2.extend(rules.domain.iter().map(|s| s.to_string()));
    p2.sort_unstable();
    if !p2.is_empty() {
        fs::write(out_dir.join(format!("{}.txt", name)), p2.join("\n") + "\n")?;
    }
    Ok(())
}

fn to_exclave(name: &str, rules: &RuleSet, out_dir: &Path) -> Result<()> {
    let out = out_dir.join(format!("{}.list", name));
    let mut p = rules.domain_suffix.iter().map(|s| format!("+.{}", s)).collect::<Vec<_>>();
    p.extend(rules.domain.iter().map(|s| s.to_string()));
    p.extend(rules.ip_cidr.iter().map(|s| s.to_string()));
    p.sort_unstable();
    if !p.is_empty() {
        fs::write(&out, p.join("\n") + "\n")?;
    }
    Ok(())
}

fn to_anywhere(name: &str, rules: &RuleSet, out_dir: &Path) -> Result<()> {
    let out = out_dir.join(format!("{}.json", name));
    let mut p = Vec::new();
    for s in &rules.domain_suffix { p.push(serde_json::json!({"type": 2, "value": s})); }
    for s in &rules.domain { p.push(serde_json::json!({"type": 3, "value": s})); }
    if !p.is_empty() {
        fs::write(&out, serde_json::to_string(&p)?)?;
    }
    Ok(())
}

fn to_egern(name: &str, rules: &RuleSet, out_dir: &Path) -> Result<()> {
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
    for s in &rules.ip_cidr { 
        egern_rules.push(serde_json::json!({if s.contains(':') { "ip_cidr6" } else { "ip_cidr" }: {"match": s, "policy": "Proxy", "no_resolve": "true"}}));
    }
    for s in &rules.ip_asn { 
        let asn_val = if s.starts_with("AS") { s.to_string() } else { format!("AS{}", s) };
        egern_rules.push(serde_json::json!({"asn": {"match": asn_val, "policy": "Proxy"}}));
    }
    
    if !egern_rules.is_empty() {
        let mut wrap = serde_json::Map::new();
        wrap.insert("rules".to_string(), serde_json::Value::Array(egern_rules));
        let yaml_str = serde_yaml::to_string(&wrap)?;
        fs::write(&out, format!("# Gins-Rules: {}\n{}", name, yaml_str))?;
    }
    Ok(())
}

fn to_text(name: &str, rules: &RuleSet, out_dir: &Path, is_ip: bool) -> Result<()> {
    let out = out_dir.join(format!("{}{}", name, if is_ip { ".ip.txt" } else { ".txt" }));
    let mut p = rules.domain_suffix.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    p.extend(rules.domain.iter().map(|s| s.to_string()));
    p.extend(rules.ip_cidr.iter().map(|s| s.to_string()));
    p.sort_unstable();
    if !p.is_empty() {
        fs::write(&out, p.join("\n") + "\n")?;
    }
    Ok(())
}
