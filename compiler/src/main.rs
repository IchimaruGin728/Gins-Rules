use anyhow::Result;
use clap::Parser;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = ".")]
    root: PathBuf,

    #[arg(short, long, default_value = "compiled")]
    output: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct Rules {
    #[serde(default)]
    domain_suffix: Vec<String>,
    #[serde(default)]
    domain: Vec<String>,
    #[serde(default)]
    domain_keyword: Vec<String>,
    #[serde(default)]
    domain_regex: Vec<String>,
    #[serde(default)]
    ip_cidr: Vec<String>,
    #[serde(default)]
    ip_asn: Vec<String>,
    #[serde(default)]
    process_name: Vec<String>,
    #[serde(default)]
    user_agent: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SingBoxRuleSet {
    version: i32,
    rules: Vec<SingBoxRule>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SingBoxRule {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    domain_suffix: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    domain: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    domain_keyword: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    domain_regex: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    ip_cidr: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnywhereRule {
    #[serde(rename = "type")]
    rule_type: i32,
    value: String,
}

#[derive(Debug, Serialize, Default)]
struct BuildStats {
    services: usize,
    rules: usize,
    #[serde(rename = "ipRules")]
    ip_rules: usize,
    #[serde(rename = "asnFiles")]
    asn_files: usize,
    srs: usize,
    mrs: usize,
    #[serde(rename = "categoryCounts")]
    category_counts: HashMap<String, usize>,
    formats: usize,
    timestamp: String,
}

struct MihomoRuleMode {
    behavior: String,
    is_empty: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let root = if args.root.is_absolute() {
        args.root.clone()
    } else {
        std::env::current_dir()?.join(args.root).canonicalize()?
    };

    let compiled_dir = root.join("compiled");
    let ruleset_dir = compiled_dir.join("ruleset");

    println!("============================================================");
    println!("  Gins-Rules Compiler (Rust Parallel Refactor)");
    println!("============================================================");

    let bin_dir = root.join("bin");
    let mihomo_path = find_binary("mihomo", &bin_dir);
    let singbox_path = find_binary("sing-box", &bin_dir);

    println!("\n  [Diagnostic] Root: {:?}", root);
    println!(
        "  sing-box: {} ({:?})",
        if singbox_path.is_some() { "✓" } else { "✗" },
        singbox_path
    );
    println!(
        "  mihomo:   {} ({:?})\n",
        if mihomo_path.is_some() { "✓" } else { "✗" },
        mihomo_path
    );

    let output_categories = vec!["proxy", "direct", "reject", "ip", "asn", "ai"];
    let format_dirs = vec![
        "singbox",
        "mihomo",
        "text",
        "quantumultx",
        "egern",
        "loon",
        "stash",
        "shadowrocket",
        "surfboard",
        "exclave",
        "surge",
        "anywhere",
    ];

    for fmt in &format_dirs {
        let fmt_path = ruleset_dir.join(fmt);
        if fmt_path.exists() {
            fs::remove_dir_all(&fmt_path)?;
        }
        fs::create_dir_all(&fmt_path)?;
        for cat in &output_categories {
            fs::create_dir_all(fmt_path.join(cat))?;
        }
    }

    let categories = vec!["proxy", "direct", "reject", "ip", "asn"];
    let mut category_merged_rules: HashMap<String, Rules> = HashMap::new();
    for cat in &output_categories {
        category_merged_rules.insert(cat.to_string(), Rules::default());
    }

    let mut stats = BuildStats::default();
    stats.formats = format_dirs.len();
    stats.timestamp = chrono::Utc::now().to_rfc3339();

    for category in categories {
        let mut rule_names: HashSet<String> = HashSet::new();
        let local_dir = root.join("source").join(category);
        let upstream_dir = root.join("source").join("upstream").join(category);

        let (actual_local, actual_upstream) = if category == "asn" {
            (
                root.join("source").join("ip"),
                root.join("source").join("upstream").join("ip"),
            )
        } else {
            (local_dir, upstream_dir)
        };

        for d in &[&actual_local, &actual_upstream] {
            if d.exists() {
                for entry in fs::read_dir(d)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "txt") {
                        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
                        if category == "ip" && name.starts_with("asn-") {
                            continue;
                        }
                        if category == "asn" && !name.starts_with("asn-") {
                            continue;
                        }
                        rule_names.insert(name);
                    }
                }
            }
        }

        let mut sorted_names: Vec<_> = rule_names.into_iter().collect();
        sorted_names.sort();

        let processed_rules: Vec<(String, Rules)> = sorted_names
            .par_iter()
            .map(|name| {
                let mut rules = Rules::default();
                let local_path = actual_local.join(format!("{}.txt", name));
                let upstream_path = actual_upstream.join(format!("{}.txt", name));

                if local_path.exists() {
                    rules = merge_rules(rules, parse_source(&local_path).unwrap_or_default());
                }
                if upstream_path.exists() {
                    rules = merge_rules(rules, parse_source(&upstream_path).unwrap_or_default());
                }

                if category != "proxy" {
                    rules = sanitize_rules(rules);
                }

                (name.clone(), rules)
            })
            .collect();

        for (name, rules) in processed_rules {
            let count = rules.domain_suffix.len()
                + rules.domain.len()
                + rules.domain_keyword.len()
                + rules.domain_regex.len()
                + rules.ip_cidr.len()
                + rules.ip_asn.len()
                + rules.process_name.len()
                + rules.user_agent.len();
            if count == 0 {
                continue;
            }

            compile_to_all_formats(
                &name,
                category,
                &rules,
                &ruleset_dir,
                &singbox_path,
                &mihomo_path,
            )?;

            let cat_rules = category_merged_rules.get_mut(category).unwrap();
            *cat_rules = merge_rules(cat_rules.clone(), rules.clone());

            if is_ai_rule_name(&name) {
                let ai_rules = category_merged_rules.get_mut("ai").unwrap();
                *ai_rules = merge_rules(ai_rules.clone(), rules.clone());
                compile_to_all_formats(
                    &name,
                    "ai",
                    &rules,
                    &ruleset_dir,
                    &singbox_path,
                    &mihomo_path,
                )?;
            }

            stats.services += 1;
            stats.rules += count;
            if category == "ip" || category == "asn" {
                stats.ip_rules += count;
            }
            if name.starts_with("asn-") {
                stats.asn_files += 1;
            }
            *stats
                .category_counts
                .entry(category.to_string())
                .or_default() += count;
            if singbox_path.is_some() {
                stats.srs += 1;
            }
            if mihomo_path.is_some() {
                stats.mrs += 1;
            }

            println!("  [{:<6}] {:<20} {} rules", category, name, count);
        }
    }

    println!("\n  [Finalizing] Generating merged rule-sets...");
    for category in output_categories {
        let rules = category_merged_rules.get(category).unwrap();
        if rules.domain_suffix.is_empty() && rules.ip_cidr.is_empty() && rules.ip_asn.is_empty() {
            continue;
        }
        compile_to_all_formats(
            category,
            category,
            rules,
            &ruleset_dir,
            &singbox_path,
            &mihomo_path,
        )?;
        println!(
            "  ✅ [{:<6}] Created full merged rule-set: {}",
            category, category
        );
    }

    generate_manifests(&ruleset_dir, &format_dirs)?;
    copy_parsers_js(&root, &compiled_dir)?;

    let summary_json = serde_json::to_string_pretty(&stats)?;
    fs::write(ruleset_dir.join("build-summary.json"), summary_json)?;

    println!(
        "\n  [DONE] Rust Refactor Progress: Full compiler implemented with parallel processing."
    );
    Ok(())
}

fn compile_to_all_formats(
    name: &str,
    category: &str,
    rules: &Rules,
    ruleset_dir: &Path,
    singbox_path: &Option<PathBuf>,
    mihomo_path: &Option<PathBuf>,
) -> Result<()> {
    let is_ip = category == "ip" || category == "asn";

    let json_path = compile_singbox_json(name, rules, &ruleset_dir.join("singbox").join(category))?;
    if let Some(sb) = singbox_path {
        compile_singbox_srs(&json_path, sb)?;
    }

    let mode = detect_mihomo_rule_mode(rules, is_ip);
    compile_mihomo_yaml(
        name,
        rules,
        &ruleset_dir.join("mihomo").join(category),
        &mode,
    )?;
    if let Some(mh) = mihomo_path {
        compile_mihomo_mrs(
            name,
            rules,
            &ruleset_dir.join("mihomo").join(category),
            is_ip,
            mh,
        )?;
    }

    compile_mihomo_yaml(
        name,
        rules,
        &ruleset_dir.join("stash").join(category),
        &mode,
    )?;
    if let Some(mh) = mihomo_path {
        compile_mihomo_mrs(
            name,
            rules,
            &ruleset_dir.join("stash").join(category),
            is_ip,
            mh,
        )?;
    }

    compile_text_list(name, rules, &ruleset_dir.join("text").join(category), is_ip)?;
    compile_quanx_list(
        name,
        rules,
        &ruleset_dir.join("quantumultx").join(category),
        is_ip,
        category,
    )?;
    compile_egern_yaml(name, rules, &ruleset_dir.join("egern").join(category))?;
    compile_loon_list(
        name,
        rules,
        &ruleset_dir.join("loon").join(category),
        true,
        ".lsr",
    )?;
    compile_loon_list(
        name,
        rules,
        &ruleset_dir.join("shadowrocket").join(category),
        true,
        ".list",
    )?;
    compile_shadowrocket_domainset(
        name,
        rules,
        &ruleset_dir.join("shadowrocket").join(category),
    )?;
    compile_loon_list(
        name,
        rules,
        &ruleset_dir.join("surge").join(category),
        true,
        ".list",
    )?;
    compile_surge_domainset(name, rules, &ruleset_dir.join("surge").join(category))?;
    compile_loon_list(
        name,
        rules,
        &ruleset_dir.join("surfboard").join(category),
        false,
        ".list",
    )?;
    compile_surfboard_domainset(name, rules, &ruleset_dir.join("surfboard").join(category))?;
    compile_exclave_route(name, rules, &ruleset_dir.join("exclave").join(category))?;
    compile_anywhere_json(name, rules, &ruleset_dir.join("anywhere").join(category))?;

    Ok(())
}

fn find_binary(name: &str, bin_dir: &Path) -> Option<PathBuf> {
    let local = bin_dir.join(name);
    if local.exists() {
        return Some(local);
    }
    if let Ok(path) = which::which(name) {
        return Some(path);
    }
    None
}

fn parse_source(path: &Path) -> Result<Rules> {
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

fn merge_rules(mut a: Rules, b: Rules) -> Rules {
    a.domain_suffix.extend(b.domain_suffix);
    a.domain.extend(b.domain);
    a.domain_keyword.extend(b.domain_keyword);
    a.domain_regex.extend(b.domain_regex);
    a.ip_cidr.extend(b.ip_cidr);
    a.ip_asn.extend(b.ip_asn);
    a.process_name.extend(b.process_name);
    a.user_agent.extend(b.user_agent);

    a.domain_suffix = unique(a.domain_suffix);
    a.domain = unique(a.domain);
    a.domain_keyword = unique(a.domain_keyword);
    a.domain_regex = unique(a.domain_regex);
    a.ip_cidr = unique(a.ip_cidr);
    a.ip_asn = unique(a.ip_asn);
    a.process_name = unique(a.process_name);
    a.user_agent = unique(a.user_agent);
    a
}

fn unique(mut v: Vec<String>) -> Vec<String> {
    let set: HashSet<_> = v.drain(..).collect();
    let mut v: Vec<_> = set.into_iter().collect();
    v.sort();
    v
}

fn sanitize_rules(mut rules: Rules) -> Rules {
    let force_proxy = vec![
        "browserleaks.com",
        "browserleaks.org",
        "ipleak.net",
        "ipleak.vip",
        "ipinfo.io",
        "ip.sb",
        "whoer.net",
        "dnsleaktest.com",
        "tiktok.com",
        "tiktokv.com",
        "tiktokcdn.com",
        "byteoversea.com",
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

fn compile_singbox_json(name: &str, rules: &Rules, out_dir: &Path) -> Result<PathBuf> {
    let rs = SingBoxRuleSet {
        version: 2,
        rules: vec![SingBoxRule {
            domain_suffix: rules.domain_suffix.clone(),
            domain: rules.domain.clone(),
            domain_keyword: rules.domain_keyword.clone(),
            domain_regex: rules.domain_regex.clone(),
            ip_cidr: rules.ip_cidr.clone(),
        }],
    };
    let path = out_dir.join(format!("{}.json", name));
    let data = serde_json::to_string_pretty(&rs)?;
    fs::write(&path, data)?;
    Ok(path)
}

fn compile_singbox_srs(json_path: &Path, singbox_path: &Path) -> Result<()> {
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

fn compile_mihomo_yaml(
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
    } else if mode.behavior == "ipcidr" {
        for cidr in &rules.ip_cidr {
            lines.push(format!("  - '{}'", cidr));
        }
    } else {
        let mut domains = rules.domain_suffix.clone();
        domains.extend(rules.domain.clone());
        let domains = unique(domains);
        for d in domains {
            if d.matches('.').count() > 5 {
                continue;
            }
            lines.push(format!("  - '{}'", d));
        }
    }
    fs::write(
        out_dir.join(format!("{}.yaml", name)),
        lines.join("\n") + "\n",
    )?;
    Ok(())
}

fn detect_mihomo_rule_mode(rules: &Rules, is_ip_category: bool) -> MihomoRuleMode {
    let has_ip = !rules.ip_cidr.is_empty();
    let has_asn = !rules.ip_asn.is_empty();
    let has_domain = !rules.domain_suffix.is_empty() || !rules.domain.is_empty();
    if has_asn {
        return MihomoRuleMode {
            behavior: "classical".to_string(),
            is_empty: false,
        };
    }
    if is_ip_category || (has_ip && !has_domain) {
        return MihomoRuleMode {
            behavior: "ipcidr".to_string(),
            is_empty: !has_ip,
        };
    }
    MihomoRuleMode {
        behavior: "domain".to_string(),
        is_empty: !has_domain,
    }
}

fn compile_mihomo_mrs(
    name: &str,
    rules: &Rules,
    out_dir: &Path,
    is_ip_category: bool,
    mihomo_path: &Path,
) -> Result<()> {
    let mode = detect_mihomo_rule_mode(rules, is_ip_category);
    if mode.is_empty {
        return Ok(());
    }
    let tmp_path = out_dir.join(format!(".{}.mrs-input.yaml", name));
    compile_mihomo_yaml(&format!(".{}.mrs-input", name), rules, out_dir, &mode)?;
    Command::new(mihomo_path)
        .args([
            "convert-ruleset",
            &mode.behavior,
            "yaml",
            tmp_path.to_str().unwrap(),
            out_dir.join(format!("{}.mrs", name)).to_str().unwrap(),
        ])
        .output()?;
    fs::remove_file(tmp_path)?;
    Ok(())
}

fn compile_text_list(name: &str, rules: &Rules, out_dir: &Path, is_ip: bool) -> Result<()> {
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

fn compile_quanx_list(
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
        lines.push(format!("host-suffix,{},{}", d, policy));
    }
    for d in &rules.domain {
        lines.push(format!("host,{},{}", d, policy));
    }
    for d in &rules.domain_keyword {
        lines.push(format!("host-keyword,{},{}", d, policy));
    }
    for cidr in &rules.ip_cidr {
        lines.push(format!(
            "{},{},{}",
            if cidr.contains(':') {
                "ip6-cidr"
            } else {
                "ip-cidr"
            },
            cidr,
            policy
        ));
    }
    for asn in &rules.ip_asn {
        lines.push(format!("ip-asn,{},{}", asn, policy));
    }
    for ua in &rules.user_agent {
        lines.push(format!("USER-AGENT,{},{}", ua, policy));
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

fn compile_egern_yaml(name: &str, rules: &Rules, out_dir: &Path) -> Result<()> {
    let mut lines = vec![
        format!("# Gins-Rules: {}", name),
        "# Optimized Egern Rule Set".to_string(),
        "".to_string(),
    ];
    let mut write_set = |key: &str, vals: &Vec<String>| {
        if vals.is_empty() {
            return;
        }
        lines.push(format!("{}:", key));
        for v in vals {
            lines.push(format!("  - \"{}\"", v));
        }
    };
    write_set("domain_suffix_set", &rules.domain_suffix);
    write_set("domain_set", &rules.domain);
    write_set("domain_keyword_set", &rules.domain_keyword);
    write_set("domain_regex_set", &rules.domain_regex);
    let (v4, v6): (Vec<_>, Vec<_>) = rules.ip_cidr.iter().partition(|c| !c.contains(':'));
    write_set("ip_cidr_set", &v4.into_iter().cloned().collect());
    write_set("ip_cidr6_set", &v6.into_iter().cloned().collect());
    write_set("ip_asn_set", &rules.ip_asn);
    write_set("user_agent_set", &rules.user_agent);
    fs::write(
        out_dir.join(format!("{}.yaml", name)),
        lines.join("\n") + "\n",
    )?;
    Ok(())
}

fn compile_loon_list(
    name: &str,
    rules: &Rules,
    out_dir: &Path,
    include_special: bool,
    suffix: &str,
) -> Result<()> {
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
    if include_special {
        for p in &rules.process_name {
            lines.push(format!("PROCESS-NAME,{}", p));
        }
        for ua in &rules.user_agent {
            lines.push(format!("USER-AGENT,{}", ua));
        }
    }
    fs::write(
        out_dir.join(format!("{}{}", name, suffix)),
        lines.join("\n") + "\n",
    )?;
    Ok(())
}

fn compile_surge_domainset(name: &str, rules: &Rules, out_dir: &Path) -> Result<()> {
    let mut lines = Vec::new();
    for d in &rules.domain_suffix {
        lines.push(format!(".{}", d));
    }
    for d in &rules.domain {
        lines.push(d.to_string());
    }
    if !lines.is_empty() {
        fs::write(
            out_dir.join(format!("{}.domainset", name)),
            lines.join("\n") + "\n",
        )?;
    }
    Ok(())
}

fn compile_shadowrocket_domainset(name: &str, rules: &Rules, out_dir: &Path) -> Result<()> {
    let mut lines = Vec::new();
    for d in &rules.domain_suffix {
        lines.push(format!(".{}", d));
    }
    for d in &rules.domain {
        lines.push(d.to_string());
    }
    if !lines.is_empty() {
        fs::write(
            out_dir.join(format!("{}.txt", name)),
            lines.join("\n") + "\n",
        )?;
    }
    Ok(())
}

fn compile_surfboard_domainset(name: &str, rules: &Rules, out_dir: &Path) -> Result<()> {
    let mut lines = Vec::new();
    for d in &rules.domain_suffix {
        lines.push(format!(".{}", d));
    }
    for d in &rules.domain {
        lines.push(d.to_string());
    }
    if !lines.is_empty() {
        fs::write(
            out_dir.join(format!("{}.txt", name)),
            lines.join("\n") + "\n",
        )?;
    }
    Ok(())
}

fn compile_exclave_route(name: &str, rules: &Rules, out_dir: &Path) -> Result<()> {
    let mut lines = Vec::new();
    for d in &rules.domain_suffix {
        lines.push(format!("domain:{}", d));
    }
    for d in &rules.domain {
        lines.push(format!("full:{}", d));
    }
    for d in &rules.domain_keyword {
        lines.push(format!("keyword:{}", d));
    }
    for d in &rules.domain_regex {
        lines.push(format!("regexp:{}", d));
    }
    for d in &rules.ip_cidr {
        lines.push(format!("ip:{}", d));
    }
    for d in &rules.ip_asn {
        lines.push(format!("asn:{}", d));
    }
    fs::write(
        out_dir.join(format!("{}.list", name)),
        lines.join("\n") + "\n",
    )?;
    Ok(())
}

fn compile_anywhere_json(name: &str, rules: &Rules, out_dir: &Path) -> Result<()> {
    let mut anywhere_rules = Vec::new();
    for cidr in &rules.ip_cidr {
        anywhere_rules.push(AnywhereRule {
            rule_type: if cidr.contains(':') { 1 } else { 0 },
            value: cidr.clone(),
        });
    }
    for suffix in &rules.domain_suffix {
        anywhere_rules.push(AnywhereRule {
            rule_type: 2,
            value: suffix.clone(),
        });
    }
    for keyword in &rules.domain_keyword {
        anywhere_rules.push(AnywhereRule {
            rule_type: 3,
            value: keyword.clone(),
        });
    }
    if !anywhere_rules.is_empty() {
        fs::write(
            out_dir.join(format!("{}.json", name)),
            serde_json::to_string_pretty(&anywhere_rules)?,
        )?;
    }
    Ok(())
}

fn generate_manifests(ruleset_dir: &Path, format_dirs: &[&str]) -> Result<()> {
    for fmt in format_dirs {
        for cat in &["proxy", "direct", "reject", "ip", "asn", "ai"] {
            let dir = ruleset_dir.join(fmt).join(cat);
            if let Ok(entries) = fs::read_dir(&dir) {
                let mut files: Vec<_> = entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.file_name().into_string().unwrap())
                    .filter(|n| n != "manifest.json")
                    .collect();
                if !files.is_empty() {
                    files.sort();
                    fs::write(
                        dir.join("manifest.json"),
                        serde_json::to_string_pretty(&files)?,
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn copy_parsers_js(root: &Path, compiled_dir: &Path) -> Result<()> {
    let dashboard_public = root.join("dashboard").join("public");
    fs::create_dir_all(&dashboard_public)?;
    for p in [
        "QX-Resource-Parser.js",
        "Loon-Resource-Parser.js",
        "geo_location_checker.js",
    ] {
        if let Ok(data) = fs::read(root.join("source").join(p)) {
            fs::write(compiled_dir.join(p), &data)?;
            fs::write(dashboard_public.join(p), &data)?;
            println!(
                "  [SUCCESS] Distributed {} to compiled/ and dashboard/public/",
                p
            );
        }
    }
    Ok(())
}

fn is_ai_rule_name(name: &str) -> bool {
    vec![
        "ai-other",
        "apple-intelligence",
        "claude",
        "copilot",
        "gemini",
        "openai",
    ]
    .contains(&name)
}
