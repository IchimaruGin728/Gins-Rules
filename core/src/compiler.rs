use crate::models::{AnywhereRule, MihomoRuleMode, Rules, SingBoxRule, SingBoxRuleSet};
use crate::parser::unique;
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn compile_singbox_json(
    name: &str,
    rules: &Rules,
    out_dir: &Path,
) -> Result<std::path::PathBuf> {
    let rs = SingBoxRuleSet {
        version: 5,
        rules: vec![SingBoxRule {
            domain_suffix: rules.domain_suffix.clone(),
            domain: rules.domain.clone(),
            domain_keyword: rules.domain_keyword.clone(),
            domain_regex: rules.domain_regex.clone(),
            ip_cidr: rules.ip_cidr.clone(),
            process_name: rules.process_name.clone(),
            user_agent: rules.user_agent.clone(),
        }],
    };
    let path = out_dir.join(format!("{}.json", name));
    fs::write(&path, serde_json::to_string_pretty(&rs)?)?;
    Ok(path)
}

pub fn compile_singbox_srs(json_path: &Path, singbox_path: &Path) -> Result<()> {
    Command::new(singbox_path)
        .args([
            "rule-set",
            "compile",
            json_path.to_str().unwrap(),
            "-o",
            json_path.with_extension("srs").to_str().unwrap(),
        ])
        .output()?;
    Ok(())
}

pub fn compile_mihomo_yaml(
    name: &str,
    rules: &Rules,
    out_dir: &Path,
    mode: &MihomoRuleMode,
) -> Result<()> {
    let mut lines = vec![
        format!("# Gins-Rules: {}", name),
        "# Auto-generated, do not edit".to_string(),
        "".to_string(),
        "payload:".to_string(),
    ];
    if mode.behavior == "classical" {
        for d in &rules.domain_suffix {
            lines.push(format!("  - 'DOMAIN-SUFFIX,{}'", d));
        }
        for d in &rules.domain {
            lines.push(format!("  - 'DOMAIN,{}'", d));
        }
        for d in &rules.domain_keyword {
            lines.push(format!("  - 'DOMAIN-KEYWORD,{}'", d));
        }
        for d in &rules.domain_regex {
            lines.push(format!("  - 'DOMAIN-REGEXP,{}'", d));
        }
        for cidr in &rules.ip_cidr {
            lines.push(format!(
                "  - '{},{}'",
                if cidr.contains(':') {
                    "IP-CIDR6"
                } else {
                    "IP-CIDR"
                },
                cidr
            ));
        }
        for asn in &rules.ip_asn {
            lines.push(format!("  - 'IP-ASN,{}'", asn));
        }
        for p in &rules.process_name {
            lines.push(format!("  - 'PROCESS-NAME,{}'", p));
        }
        for u in &rules.user_agent {
            lines.push(format!("  - 'USER-AGENT,{}'", u));
        }
    } else if mode.behavior == "ipcidr" {
        for cidr in &rules.ip_cidr {
            lines.push(format!("  - '{}'", cidr));
        }
    } else {
        let mut domains = rules.domain_suffix.clone();
        domains.extend(rules.domain.clone());
        for d in unique(domains) {
            if d.matches('.').count() <= 5 {
                lines.push(format!("  - '{}'", d));
            }
        }
    }
    fs::write(
        out_dir.join(format!("{}.yaml", name)),
        lines.join("\n") + "\n",
    )?;
    Ok(())
}

pub fn detect_mihomo_rule_mode(rules: &Rules, is_ip_category: bool) -> MihomoRuleMode {
    let has_ip = !rules.ip_cidr.is_empty();
    let has_asn = !rules.ip_asn.is_empty();
    let has_domain = !rules.domain_suffix.is_empty()
        || !rules.domain.is_empty()
        || !rules.domain_keyword.is_empty()
        || !rules.domain_regex.is_empty();
    let has_other = !rules.process_name.is_empty() || !rules.user_agent.is_empty();

    let has_classical_only = !rules.domain_keyword.is_empty() || !rules.domain_regex.is_empty();

    if has_other || has_asn || (has_ip && has_domain) || has_classical_only {
        return MihomoRuleMode {
            behavior: "classical".to_string(),
        };
    }
    if is_ip_category || (has_ip && !has_domain) {
        return MihomoRuleMode {
            behavior: "ipcidr".to_string(),
        };
    }
    MihomoRuleMode {
        behavior: "domain".to_string(),
    }
}

pub fn compile_mihomo_mrs_split(
    name: &str,
    rules: &Rules,
    out_dir: &Path,
    is_ip_category: bool,
    mihomo_path: &Path,
) -> Result<()> {
    // 1. Separate rules into Non-IP and IP parts
    let mut non_ip_rules = Rules::default();
    non_ip_rules.domain_suffix = rules.domain_suffix.clone();
    non_ip_rules.domain = rules.domain.clone();
    non_ip_rules.domain_keyword = rules.domain_keyword.clone();
    non_ip_rules.domain_regex = rules.domain_regex.clone();
    non_ip_rules.process_name = rules.process_name.clone();
    non_ip_rules.user_agent = rules.user_agent.clone();

    let mut ip_rules = Rules::default();
    ip_rules.ip_cidr = rules.ip_cidr.clone();
    ip_rules.ip_asn = rules.ip_asn.clone();

    let has_non_ip = !non_ip_rules.is_empty();
    let has_ip = !ip_rules.is_empty();

    // 2. Generate MRS files based on content
    if has_non_ip && has_ip {
        // CASE: Mixed content - SPLIT
        // Main MRS (Domains/Process/etc)
        let mode = detect_mihomo_rule_mode(&non_ip_rules, false);
        let tmp_name = format!(".{}.mrs-main", name);
        compile_mihomo_yaml(&tmp_name, &non_ip_rules, out_dir, &mode)?;
        let _ = Command::new(mihomo_path)
            .args([
                "convert-ruleset",
                &mode.behavior,
                "yaml",
                out_dir.join(format!("{}.yaml", tmp_name)).to_str().unwrap(),
                out_dir.join(format!("{}.mrs", name)).to_str().unwrap(),
            ])
            .output()?;
        let _ = fs::remove_file(out_dir.join(format!("{}.yaml", tmp_name)));

        // IP MRS (Split out)
        let mode_ip = detect_mihomo_rule_mode(&ip_rules, true);
        let tmp_name_ip = format!(".{}.mrs-ip", name);
        compile_mihomo_yaml(&tmp_name_ip, &ip_rules, out_dir, &mode_ip)?;
        let _ = Command::new(mihomo_path)
            .args([
                "convert-ruleset",
                &mode_ip.behavior,
                "yaml",
                out_dir
                    .join(format!("{}.yaml", tmp_name_ip))
                    .to_str()
                    .unwrap(),
                out_dir.join(format!("{}-ip.mrs", name)).to_str().unwrap(),
            ])
            .output()?;
        let _ = fs::remove_file(out_dir.join(format!("{}.yaml", tmp_name_ip)));
    } else if has_non_ip {
        // CASE: Domain only - SINGLE
        let mode = detect_mihomo_rule_mode(&non_ip_rules, false);
        let tmp_name = format!(".{}.mrs-single", name);
        compile_mihomo_yaml(&tmp_name, &non_ip_rules, out_dir, &mode)?;
        let _ = Command::new(mihomo_path)
            .args([
                "convert-ruleset",
                &mode.behavior,
                "yaml",
                out_dir.join(format!("{}.yaml", tmp_name)).to_str().unwrap(),
                out_dir.join(format!("{}.mrs", name)).to_str().unwrap(),
            ])
            .output()?;
        let _ = fs::remove_file(out_dir.join(format!("{}.yaml", tmp_name)));
    } else if has_ip {
        // CASE: IP only - SINGLE
        let mode = detect_mihomo_rule_mode(&ip_rules, true);
        let tmp_name = format!(".{}.mrs-single", name);
        compile_mihomo_yaml(&tmp_name, &ip_rules, out_dir, &mode)?;

        // If it's an IP category, we name it name.mrs
        // If it's a proxy category but only has IPs, we still name it name.mrs to avoid 404
        let output_mrs = if is_ip_category {
            out_dir.join(format!("{}.mrs", name))
        } else {
            // For proxy rules that are purely IP, generate both for safety?
            // Actually, just generate the main name.mrs so the link works.
            out_dir.join(format!("{}.mrs", name))
        };

        let _ = Command::new(mihomo_path)
            .args([
                "convert-ruleset",
                &mode.behavior,
                "yaml",
                out_dir.join(format!("{}.yaml", tmp_name)).to_str().unwrap(),
                output_mrs.to_str().unwrap(),
            ])
            .output()?;
        let _ = fs::remove_file(out_dir.join(format!("{}.yaml", tmp_name)));

        // If not in IP category, also generate the -ip.mrs version so the virtual entry works
        if !is_ip_category {
            fs::copy(output_mrs, out_dir.join(format!("{}-ip.mrs", name)))?;
        }
    }

    Ok(())
}

pub fn compile_text_list(name: &str, rules: &Rules, out_dir: &Path, is_ip: bool) -> Result<()> {
    let mut lines = Vec::new();
    for d in &rules.domain_suffix {
        lines.push(format!("DOMAIN-SUFFIX,{}", d));
    }
    for d in &rules.domain {
        lines.push(format!("DOMAIN,{}", d));
    }
    for d in &rules.domain_keyword {
        lines.push(format!("DOMAIN-KEYWORD,{}", d));
    }
    for cidr in &rules.ip_cidr {
        lines.push(format!(
            "{},{}",
            if cidr.contains(':') {
                "IP-CIDR6"
            } else {
                "IP-CIDR"
            },
            cidr
        ));
    }
    for asn in &rules.ip_asn {
        lines.push(format!("IP-ASN,{}", asn));
    }
    fs::write(
        out_dir.join(format!(
            "{}{}",
            name,
            if is_ip { ".ip.list" } else { ".list" }
        )),
        lines.join("\n") + "\n",
    )?;
    Ok(())
}

pub fn compile_quanx_list(
    name: &str,
    rules: &Rules,
    out_dir: &Path,
    is_ip: bool,
    category: &str,
) -> Result<()> {
    let policy = match category {
        "direct" => "Direct",
        "reject" => "Reject",
        _ => "Proxy",
    };
    let mut lines = Vec::new();
    for d in &rules.domain_suffix {
        lines.push(format!("HOST-SUFFIX,{},{}", d, policy));
    }
    for d in &rules.domain {
        lines.push(format!("HOST,{},{}", d, policy));
    }
    for d in &rules.domain_keyword {
        lines.push(format!("HOST-KEYWORD,{},{}", d, policy));
    }
    for cidr in &rules.ip_cidr {
        lines.push(format!(
            "{},{},{}",
            if cidr.contains(':') {
                "IP6-CIDR"
            } else {
                "IP-CIDR"
            },
            cidr,
            policy
        ));
    }
    for u in &rules.user_agent {
        lines.push(format!("USER-AGENT,{},{}", u, policy));
    }
    fs::write(
        out_dir.join(format!(
            "{}{}",
            name,
            if is_ip { ".ip.list" } else { ".list" }
        )),
        lines.join("\n") + "\n",
    )?;
    Ok(())
}

pub fn compile_egern_yaml(name: &str, rules: &Rules, out_dir: &Path, category: &str) -> Result<()> {
    let policy = match category {
        "direct" => "Direct",
        "reject" => "Reject",
        _ => "Proxy",
    };
    let mut lines = vec!["rules:".to_string()];
    for d in &rules.domain_suffix {
        lines.push(format!(
            "  - domain_suffix: {{ match: \"{}\", policy: {} }}",
            d, policy
        ));
    }
    for d in &rules.domain {
        lines.push(format!(
            "  - domain: {{ match: \"{}\", policy: {} }}",
            d, policy
        ));
    }
    for cidr in &rules.ip_cidr {
        let p = if cidr.contains(':') {
            "ip_cidr6"
        } else {
            "ip_cidr"
        };
        lines.push(format!(
            "  - {}: {{ match: \"{}\", policy: {} }}",
            p, cidr, policy
        ));
    }
    for p in &rules.process_name {
        lines.push(format!(
            "  - process_name: {{ match: \"{}\", policy: {} }}",
            p, policy
        ));
    }
    for u in &rules.user_agent {
        lines.push(format!(
            "  - user_agent: {{ match: \"{}\", policy: {} }}",
            u, policy
        ));
    }
    fs::write(
        out_dir.join(format!("{}.yaml", name)),
        lines.join("\n") + "\n",
    )?;
    Ok(())
}

pub fn compile_loon_list(
    name: &str,
    rules: &Rules,
    out_dir: &Path,
    is_suffix: bool,
    ext: &str,
    domain_prefix: &str,
    ip6_prefix: &str,
    include_other: bool,
) -> Result<()> {
    let mut lines = Vec::new();
    for d in &rules.domain_suffix {
        lines.push(if is_suffix {
            format!("{}-SUFFIX,{}", domain_prefix, d)
        } else {
            d.clone()
        });
    }
    for d in &rules.domain {
        lines.push(format!("{},{}", domain_prefix, d));
    }
    for cidr in &rules.ip_cidr {
        lines.push(format!(
            "{},{}",
            if cidr.contains(':') {
                ip6_prefix
            } else {
                "IP-CIDR"
            },
            cidr
        ));
    }
    if include_other {
        for p in &rules.process_name {
            lines.push(format!("PROCESS-NAME,{}", p));
        }
        for u in &rules.user_agent {
            lines.push(format!("USER-AGENT,{}", u));
        }
    }
    fs::write(
        out_dir.join(format!("{}{}", name, ext)),
        lines.join("\n") + "\n",
    )?;
    Ok(())
}

pub fn compile_shadowrocket_domainset(name: &str, rules: &Rules, out_dir: &Path) -> Result<()> {
    let mut lines = Vec::new();
    for d in &rules.domain_suffix {
        lines.push(format!(".{}", d));
    }
    for d in &rules.domain {
        lines.push(d.clone());
    }
    if !lines.is_empty() {
        fs::write(
            out_dir.join(format!("{}.domainset", name)),
            lines.join("\n") + "\n",
        )?;
    }
    Ok(())
}

pub fn compile_surge_domainset(name: &str, rules: &Rules, out_dir: &Path) -> Result<()> {
    let mut lines = Vec::new();
    for d in &rules.domain_suffix {
        lines.push(format!(".{}", d));
    }
    for d in &rules.domain {
        lines.push(d.clone());
    }
    if !lines.is_empty() {
        fs::write(
            out_dir.join(format!("{}.domainset", name)),
            lines.join("\n") + "\n",
        )?;
    }
    Ok(())
}

pub fn compile_surfboard_domainset(name: &str, rules: &Rules, out_dir: &Path) -> Result<()> {
    let mut lines = Vec::new();
    for d in &rules.domain_suffix {
        lines.push(format!(".{}", d));
    }
    for d in &rules.domain {
        lines.push(d.clone());
    }
    if !lines.is_empty() {
        fs::write(
            out_dir.join(format!("{}.domainset", name)),
            lines.join("\n") + "\n",
        )?;
    }
    Ok(())
}

pub fn compile_exclave_route(name: &str, rules: &Rules, out_dir: &Path) -> Result<()> {
    let mut routes = Vec::new();
    for cidr in &rules.ip_cidr {
        routes.push(cidr.clone());
    }
    for d in &rules.domain_suffix {
        routes.push(format!("+.{}", d));
    }
    for d in &rules.domain {
        routes.push(d.clone());
    }
    if !routes.is_empty() {
        fs::write(
            out_dir.join(format!("{}.json", name)),
            serde_json::to_string_pretty(&routes)?,
        )?;
    }
    Ok(())
}

pub fn compile_anywhere_json(name: &str, rules: &Rules, out_dir: &Path) -> Result<()> {
    let mut any_rules = Vec::new();
    for d in &rules.domain_suffix {
        any_rules.push(AnywhereRule {
            rule_type: 2,
            value: d.clone(),
        });
    }
    for d in &rules.domain {
        any_rules.push(AnywhereRule {
            rule_type: 3,
            value: d.clone(),
        });
    }
    if !any_rules.is_empty() {
        fs::write(
            out_dir.join(format!("{}.json", name)),
            serde_json::to_string_pretty(&any_rules)?,
        )?;
    }
    Ok(())
}
