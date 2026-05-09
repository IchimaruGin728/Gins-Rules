use anyhow::Result;
use clap::Parser;
use maxminddb_writer::metadata::IpVersion;
use maxminddb_writer::paths::IpAddrWithMask;
use maxminddb_writer::Database;
use prost::Message;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::net::IpAddr;
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

#[derive(Serialize, Deserialize)]
struct AsnPrefixRecord {
    asn: u32,
    cidr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    org: Option<String>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GeoIp {
    #[prost(string, tag = "1")]
    pub country_code: String,
    #[prost(message, repeated, tag = "2")]
    pub cidr: Vec<Cidr>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Cidr {
    #[prost(bytes = "vec", tag = "1")]
    pub ip: Vec<u8>,
    #[prost(uint32, tag = "2")]
    pub prefix: u32,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GeoIpList {
    #[prost(message, repeated, tag = "1")]
    pub entry: Vec<GeoIp>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GeoSite {
    #[prost(string, tag = "1")]
    pub country_code: String,
    #[prost(message, repeated, tag = "2")]
    pub domain: Vec<Domain>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Domain {
    #[prost(enumeration = "DomainType", tag = "1")]
    pub r#type: i32,
    #[prost(string, tag = "2")]
    pub value: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum DomainType {
    Plain = 0,
    Regex = 1,
    Domain = 2,
    Full = 3,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GeoSiteList {
    #[prost(message, repeated, tag = "1")]
    pub entry: Vec<GeoSite>,
}

#[derive(Serialize)]
struct CountryRecord {
    country: CountryIso,
}

#[derive(Serialize)]
struct CountryIso {
    iso_code: String,
}

#[derive(Serialize)]
struct AsnRecord {
    autonomous_system_number: u32,
    autonomous_system_organization: String,
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
        println!("  [Diagnostic] Handling format dir: {}", fmt);
        if fmt_path.exists() {
            println!("  [Diagnostic] Removing format dir: {}", fmt);
            fs::remove_dir_all(&fmt_path)?;
        }
        println!("  [Diagnostic] Creating format dir: {}", fmt);
        fs::create_dir_all(&fmt_path)?;
        for cat in &output_categories {
            fs::create_dir_all(fmt_path.join(cat))?;
        }
    }
    println!("  [Diagnostic] Directories created");

    let categories = vec!["proxy", "direct", "reject", "ip", "asn"];
    let mut category_merged_rules: HashMap<String, Rules> = HashMap::new();
    for cat in &output_categories {
        category_merged_rules.insert(cat.to_string(), Rules::default());
    }

    let mut stats = BuildStats::default();
    stats.formats = format_dirs.len();
    stats.timestamp = chrono::Utc::now().to_rfc3339();

    let mut all_rules: HashMap<String, HashMap<String, Rules>> = HashMap::new();

    for category in categories {
        println!("  [Diagnostic] Processing category: {}", category);
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
        println!(
            "  [Diagnostic] Found {} rules in category: {}",
            sorted_names.len(),
            category
        );

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

        println!(
            "  [Diagnostic] Parsed and merged rules for category: {}",
            category
        );
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
            all_rules
                .entry(category.to_string())
                .or_default()
                .insert(name.clone(), rules.clone());
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
    }
    pack_binary_assets(&all_rules, &compiled_dir, &root)?;
    generate_manifests(&ruleset_dir, &format_dirs)?;
    copy_parsers_js(&root, &compiled_dir)?;
    fs::write(
        compiled_dir.join("build-summary.json"),
        serde_json::to_string_pretty(&stats)?,
    )?;
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
    println!(
        "    [Diagnostic] compile_to_all_formats starting for: {}",
        name
    );
    let is_ip = category == "ip" || category == "asn";
    let json_path = compile_singbox_json(name, rules, &ruleset_dir.join("singbox").join(category))?;
    if let Some(sb) = singbox_path {
        println!("    [Diagnostic] Compiling singbox srs for: {}", name);
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
        println!("    [Diagnostic] Compiling mihomo mrs for: {}", name);
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
        println!("    [Diagnostic] Compiling stash mrs for: {}", name);
        compile_mihomo_mrs(
            name,
            rules,
            &ruleset_dir.join("stash").join(category),
            is_ip,
            mh,
        )?;
    }

    println!("    [Diagnostic] Compiling text lists for: {}", name);
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

    println!(
        "    [Diagnostic] compile_to_all_formats finished for: {}",
        name
    );
    Ok(())
}

fn pack_binary_assets(
    all_rules: &HashMap<String, HashMap<String, Rules>>,
    out_dir: &Path,
    root: &Path,
) -> Result<()> {
    let mut geosite_list = GeoSiteList::default();
    let mut geoip_list = GeoIpList::default();
    let mut mmdb_geoip = Database::default();
    mmdb_geoip.metadata.database_type = "GeoLite2-Country".to_string();
    mmdb_geoip.metadata.languages = vec!["en".to_string()];
    mmdb_geoip.metadata.ip_version = IpVersion::V6;
    mmdb_geoip.metadata.binary_format_major_version = 2;
    mmdb_geoip.metadata.build_epoch = chrono::Utc::now().timestamp() as u64;

    let mut mmdb_geoasn = Database::default();
    mmdb_geoasn.metadata.database_type = "GeoLite2-ASN".to_string();
    mmdb_geoasn.metadata.languages = vec!["en".to_string()];
    mmdb_geoasn.metadata.ip_version = IpVersion::V6;
    mmdb_geoasn.metadata.binary_format_major_version = 2;
    mmdb_geoasn.metadata.build_epoch = chrono::Utc::now().timestamp() as u64;
    let asn_prefix_index_path = root.join("compiled").join("asn-prefix-index.json");
    let asn_prefix_index: HashMap<String, Vec<AsnPrefixRecord>> = if asn_prefix_index_path.exists()
    {
        serde_json::from_str(&fs::read_to_string(asn_prefix_index_path)?)?
    } else {
        HashMap::new()
    };
    for (category, rules_map) in all_rules {
        for (name, rules) in rules_map {
            let tag = name.to_uppercase();
            if category != "ip" && category != "asn" {
                let mut site = GeoSite {
                    country_code: tag.clone(),
                    domain: Vec::new(),
                };
                for d in &rules.domain_suffix {
                    site.domain.push(Domain {
                        r#type: DomainType::Domain as i32,
                        value: d.clone(),
                    });
                }
                for d in &rules.domain {
                    site.domain.push(Domain {
                        r#type: DomainType::Full as i32,
                        value: d.clone(),
                    });
                }
                for d in &rules.domain_keyword {
                    site.domain.push(Domain {
                        r#type: DomainType::Plain as i32,
                        value: d.clone(),
                    });
                }
                for d in &rules.domain_regex {
                    site.domain.push(Domain {
                        r#type: DomainType::Regex as i32,
                        value: d.clone(),
                    });
                }
                if !site.domain.is_empty() {
                    geosite_list.entry.push(site);
                }
            }
            if category == "ip" || category == "asn" {
                let mut cidrs = rules.ip_cidr.clone();
                if category == "asn" {
                    if let Some(records) = asn_prefix_index.get(name) {
                        for r in records {
                            cidrs.push(r.cidr.clone());
                        }
                    }
                }
                cidrs.sort();
                cidrs.dedup();

                let mut geoip_tag = tag.clone();
                if category == "asn" {
                    geoip_tag = name.strip_prefix("asn-").unwrap_or(name).to_uppercase();
                }

                if !cidrs.is_empty() {
                    let mut geo_ip = GeoIp {
                        country_code: geoip_tag.clone(),
                        cidr: Vec::new(),
                    };
                    for cidr_str in &cidrs {
                        if let Ok(net) = cidr_str.parse::<IpAddrWithMask>() {
                            let ip_bytes = match net.addr {
                                IpAddr::V4(a) => a.octets().to_vec(),
                                IpAddr::V6(a) => a.octets().to_vec(),
                            };
                            geo_ip.cidr.push(Cidr {
                                ip: ip_bytes,
                                prefix: net.mask as u32,
                            });

                            let mmdb_net = match net.addr {
                                IpAddr::V4(a) => IpAddrWithMask::new(
                                    IpAddr::V6(a.to_ipv6_mapped()),
                                    net.mask + 96,
                                ),
                                IpAddr::V6(_) => net,
                            };

                            let data_ref = mmdb_geoip
                                .insert_value(&CountryRecord {
                                    country: CountryIso {
                                        iso_code: geoip_tag.clone(),
                                    },
                                })
                                .unwrap();
                            mmdb_geoip.insert_node(mmdb_net, data_ref);
                        }
                    }
                    if !geo_ip.cidr.is_empty() {
                        geoip_list.entry.push(geo_ip);
                    }
                }
            }
            if category == "asn" {
                if let Some(records) = asn_prefix_index.get(name) {
                    for r in records {
                        if let Ok(net) = r.cidr.parse::<IpAddrWithMask>() {
                            let mmdb_net = match net.addr {
                                IpAddr::V4(a) => IpAddrWithMask::new(
                                    IpAddr::V6(a.to_ipv6_mapped()),
                                    net.mask + 96,
                                ),
                                IpAddr::V6(_) => net,
                            };
                            let org = r.org.clone().unwrap_or_else(|| format!("AS{}", r.asn));
                            let data_ref = mmdb_geoasn
                                .insert_value(&AsnRecord {
                                    autonomous_system_number: r.asn,
                                    autonomous_system_organization: org,
                                })
                                .unwrap();
                            mmdb_geoasn.insert_node(mmdb_net, data_ref);
                        }
                    }
                }
            }
        }
    }
    let xray_dir = out_dir.join("xray");
    fs::create_dir_all(&xray_dir)?;
    fs::write(xray_dir.join("geosite.dat"), geosite_list.encode_to_vec())?;
    fs::write(xray_dir.join("geoip.dat"), geoip_list.encode_to_vec())?;
    let out_geoip = fs::File::create(out_dir.join("geoip.mmdb"))?;
    mmdb_geoip.write_to(out_geoip).unwrap();
    let out_geoasn = fs::File::create(out_dir.join("geoasn.mmdb"))?;
    mmdb_geoasn.write_to(out_geoasn).unwrap();
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
    fs::write(&path, serde_json::to_string_pretty(&rs)?)?;
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

fn detect_mihomo_rule_mode(rules: &Rules, is_ip_category: bool) -> MihomoRuleMode {
    let has_ip = !rules.ip_cidr.is_empty();
    let has_asn = !rules.ip_asn.is_empty();
    let has_domain = !rules.domain_suffix.is_empty()
        || !rules.domain.is_empty()
        || !rules.domain_keyword.is_empty()
        || !rules.domain_regex.is_empty();
    let has_other = !rules.process_name.is_empty() || !rules.user_agent.is_empty();

    if has_other || has_asn || (has_ip && has_domain) {
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
