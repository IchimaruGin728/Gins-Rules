use anyhow::Result;
use clap::Parser;
use gins_rules_core::*;
use maxminddb_writer::metadata::IpVersion;
use maxminddb_writer::paths::IpAddrWithMask;
use maxminddb_writer::Database;
use prost::Message;
use rayon::prelude::*;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = ".")]
    root: PathBuf,

    #[arg(short, long, default_value = "compiled")]
    output: PathBuf,
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

    if let Some(ref path) = mihomo_path {
        let out = Command::new(path).arg("-v").output()?;
        println!(
            "  [Binary] Mihomo version: {}",
            String::from_utf8_lossy(&out.stdout).trim()
        );
    }
    if let Some(ref path) = singbox_path {
        let out = Command::new(path).arg("version").output()?;
        println!(
            "  [Binary] Sing-box version: {}",
            String::from_utf8_lossy(&out.stdout)
                .trim()
                .split('\n')
                .next()
                .unwrap_or("")
        );
    }

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

        let sorted_names: Vec<_> = rule_names.into_iter().collect();
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

        let mut category_rules_map = HashMap::new();
        for (name, rules) in processed_rules {
            let count = rules.count();

            if count > 0 {
                category_rules_map.insert(name.clone(), rules.clone());
                let skip_ruleset =
                    category == "ip" && ["de", "fr", "kr", "us"].contains(&name.as_str());
                if !skip_ruleset {
                    compile_to_all_formats(
                        &name,
                        category,
                        &rules,
                        &ruleset_dir,
                        &singbox_path,
                        &mihomo_path,
                    )?;
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
                }
            }
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
        }
        all_rules.insert(category.to_string(), category_rules_map);
    }

    // Process aggregate bundles (proxy, direct, ai, reject)
    for category in output_categories {
        let rules = category_merged_rules.get(category).unwrap();
        if rules.is_empty() {
            continue;
        }
        let bundle_name = category.to_string();
        compile_to_all_formats(
            &bundle_name,
            category,
            rules,
            &ruleset_dir,
            &singbox_path,
            &mihomo_path,
        )?;
    }

    pack_binary_assets(&all_rules, &ruleset_dir, &root)?;
    generate_manifests(&ruleset_dir, &format_dirs)?;
    copy_parsers_js(&root, &compiled_dir)?;
    fs::write(
        ruleset_dir.join("build-summary.json"),
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
    let is_ip = category == "ip" || category == "asn";
    if let Some(sb) = singbox_path {
        let json_path =
            compile_singbox_json(name, rules, &ruleset_dir.join("singbox").join(category))?;
        compile_singbox_srs(&json_path, sb)?;
        fs::remove_file(json_path)?;
    }

    let mode = detect_mihomo_rule_mode(rules, is_ip);
    // Classical YAML for Mihomo AND Stash as fallback
    compile_mihomo_yaml(
        name,
        rules,
        &ruleset_dir.join("mihomo").join(category),
        &mode,
    )?;

    let stash_mode = MihomoRuleMode {
        behavior: "classical".to_string(),
    };
    compile_mihomo_yaml(
        name,
        rules,
        &ruleset_dir.join("stash").join(category),
        &stash_mode,
    )?;

    if let Some(mh) = mihomo_path {
        compile_mihomo_mrs_split(
            name,
            rules,
            &ruleset_dir.join("stash").join(category),
            is_ip,
            mh,
        )?;
        compile_mihomo_mrs_split(
            name,
            rules,
            &ruleset_dir.join("mihomo").join(category),
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
    compile_egern_yaml(
        name,
        rules,
        &ruleset_dir.join("egern").join(category),
        category,
    )?;

    compile_loon_list(
        name,
        rules,
        &ruleset_dir.join("loon").join(category),
        true,
        ".lsr",
        "DOMAIN",
        "IP-CIDR6",
        false,
    )?;
    compile_loon_list(
        name,
        rules,
        &ruleset_dir.join("shadowrocket").join(category),
        true,
        ".list",
        "DOMAIN",
        "IP-CIDR6",
        true,
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
        "DOMAIN",
        "IP-CIDR6",
        true,
    )?;
    compile_surge_domainset(name, rules, &ruleset_dir.join("surge").join(category))?;

    compile_loon_list(
        name,
        rules,
        &ruleset_dir.join("surfboard").join(category),
        false,
        ".list",
        "DOMAIN",
        "IP-CIDR6",
        false,
    )?;
    compile_surfboard_domainset(name, rules, &ruleset_dir.join("surfboard").join(category))?;

    compile_exclave_route(name, rules, &ruleset_dir.join("exclave").join(category))?;
    compile_anywhere_json(name, rules, &ruleset_dir.join("anywhere").join(category))?;
    Ok(())
}

fn pack_binary_assets(
    all_rules: &HashMap<String, HashMap<String, Rules>>,
    out_dir: &Path,
    root: &Path,
) -> Result<()> {
    let mut geosite_list = GeoSiteList::default();
    let mut geoip_list = GeoIpList::default();
    let mut geoasn_list = GeoIpList::default();
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
    let compiled_dir = root.join("compiled");
    let country_index_path = compiled_dir.join("full-country-index.json");
    let full_country_index: BTreeMap<String, Vec<String>> = if country_index_path.exists() {
        serde_json::from_str(&fs::read_to_string(country_index_path)?)?
    } else {
        BTreeMap::new()
    };
    let asn_index_path = compiled_dir.join("full-asn-index.json");
    let full_asn_index: Vec<AsnPrefixRecord> = if asn_index_path.exists() {
        let content = fs::read_to_string(&asn_index_path)?;
        serde_json::from_str(&content)?
    } else {
        Vec::new()
    };
    let mut geoip_value_cache = HashMap::new();
    let mut geoasn_value_cache = HashMap::new();
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
            if category == "ip" {
                let mut geo_ip = GeoIp {
                    country_code: tag.clone(),
                    cidr: Vec::new(),
                };
                let data_ref = *geoip_value_cache.entry(tag.clone()).or_insert_with(|| {
                    mmdb_geoip
                        .insert_value(&CountryRecord {
                            country: CountryIso {
                                iso_code: tag.clone(),
                            },
                        })
                        .unwrap()
                });
                for cidr_str in &rules.ip_cidr {
                    if let Ok(net) = cidr_str.parse::<IpAddrWithMask>() {
                        geo_ip.cidr.push(Cidr {
                            ip: match net.addr {
                                IpAddr::V4(a) => a.octets().to_vec(),
                                IpAddr::V6(a) => a.octets().to_vec(),
                            },
                            prefix: net.mask as u32,
                        });
                        let mmdb_net = match net.addr {
                            IpAddr::V4(a) => {
                                IpAddrWithMask::new(IpAddr::V6(a.to_ipv6_mapped()), net.mask + 96)
                            }
                            IpAddr::V6(_) => net,
                        };
                        mmdb_geoip.insert_node(mmdb_net, data_ref);
                    }
                }
                if !geo_ip.cidr.is_empty() {
                    geoip_list.entry.push(geo_ip);
                }
            }
            if category == "asn" {
                let mut geo_asn = GeoIp {
                    country_code: tag.clone(),
                    cidr: Vec::new(),
                };
                for cidr_str in &rules.ip_cidr {
                    if let Ok(net) = cidr_str.parse::<IpAddrWithMask>() {
                        geo_asn.cidr.push(Cidr {
                            ip: match net.addr {
                                IpAddr::V4(a) => a.octets().to_vec(),
                                IpAddr::V6(a) => a.octets().to_vec(),
                            },
                            prefix: net.mask as u32,
                        });
                    }
                }
                if !geo_asn.cidr.is_empty() {
                    geoasn_list.entry.push(geo_asn);
                }
            }
        }
    }
    for (code, cidrs) in full_country_index {
        let code_upper = code.to_uppercase();
        let data_ref = *geoip_value_cache
            .entry(code_upper.clone())
            .or_insert_with(|| {
                mmdb_geoip
                    .insert_value(&CountryRecord {
                        country: CountryIso {
                            iso_code: code_upper.clone(),
                        },
                    })
                    .unwrap()
            });
        let mut geo_ip_dat = GeoIp {
            country_code: code_upper.clone(),
            cidr: Vec::new(),
        };
        for cidr_str in cidrs {
            if let Ok(net) = cidr_str.parse::<IpAddrWithMask>() {
                geo_ip_dat.cidr.push(Cidr {
                    ip: match net.addr {
                        IpAddr::V4(a) => a.octets().to_vec(),
                        IpAddr::V6(a) => a.octets().to_vec(),
                    },
                    prefix: net.mask as u32,
                });
                let mmdb_net = match net.addr {
                    IpAddr::V4(a) => {
                        IpAddrWithMask::new(IpAddr::V6(a.to_ipv6_mapped()), net.mask + 96)
                    }
                    IpAddr::V6(_) => net,
                };
                mmdb_geoip.insert_node(mmdb_net, data_ref);
            }
        }
        geoip_list.entry.push(geo_ip_dat);
    }
    let mut dat_asn_map: BTreeMap<u32, Vec<Cidr>> = BTreeMap::new();
    for r in full_asn_index {
        if let Ok(net) = r.cidr.parse::<IpAddrWithMask>() {
            let org = r.org.clone().unwrap_or_else(|| format!("AS{}", r.asn));
            let cache_key = format!("{}-{}", r.asn, org);
            let data_ref = *geoasn_value_cache.entry(cache_key).or_insert_with(|| {
                mmdb_geoasn
                    .insert_value(&AsnRecord {
                        autonomous_system_number: r.asn,
                        autonomous_system_organization: org,
                    })
                    .unwrap()
            });
            let mmdb_net = match net.addr {
                IpAddr::V4(a) => IpAddrWithMask::new(IpAddr::V6(a.to_ipv6_mapped()), net.mask + 96),
                IpAddr::V6(_) => net,
            };
            mmdb_geoasn.insert_node(mmdb_net, data_ref);
            dat_asn_map.entry(r.asn).or_default().push(Cidr {
                ip: match net.addr {
                    IpAddr::V4(a) => a.octets().to_vec(),
                    IpAddr::V6(a) => a.octets().to_vec(),
                },
                prefix: net.mask as u32,
            });
        }
    }
    let xray_dir = out_dir.join("xray");
    fs::create_dir_all(&xray_dir)?;
    fs::write(xray_dir.join("geosite.dat"), geosite_list.encode_to_vec())?;
    fs::write(xray_dir.join("geoip.dat"), geoip_list.encode_to_vec())?;
    fs::write(xray_dir.join("geoasn.dat"), geoasn_list.encode_to_vec())?;
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

fn generate_manifests(ruleset_dir: &Path, format_dirs: &[&str]) -> Result<()> {
    for fmt in format_dirs {
        let fmt_path = ruleset_dir.join(fmt);
        let mut manifest: HashMap<String, Vec<String>> = HashMap::new();
        let categories = ["proxy", "direct", "reject", "ip", "asn", "ai"];
        for cat in &categories {
            let cat_path = fmt_path.join(cat);
            if cat_path.exists() {
                let mut files = Vec::new();
                for entry in fs::read_dir(cat_path)? {
                    let entry = entry?;
                    if entry.path().is_file() {
                        files.push(entry.file_name().to_str().unwrap().to_string());
                    }
                }
                files.sort();
                manifest.insert(cat.to_string(), files);
            }
        }
        fs::write(
            fmt_path.join("manifest.json"),
            serde_json::to_string_pretty(&manifest)?,
        )?;
    }
    Ok(())
}

fn copy_parsers_js(root: &Path, compiled_dir: &Path) -> Result<()> {
    let source_dir = root.join("source");
    let dashboard_public = root.join("dashboard").join("public");
    let files = [
        "QX-Resource-Parser.js",
        "Loon-Resource-Parser.js",
        "geo_location_checker.js",
    ];
    for file in &files {
        let src = source_dir.join(file);
        if src.exists() {
            fs::copy(&src, compiled_dir.join(file))?;
            if dashboard_public.exists() {
                fs::copy(&src, dashboard_public.join(file))?;
            }
            println!(
                "  [SUCCESS] Distributed {} to compiled/ and dashboard/public/",
                file
            );
        }
    }
    Ok(())
}

fn is_ai_rule_name(name: &str) -> bool {
    let ai_keywords = [
        "openai",
        "claude",
        "gemini",
        "copilot",
        "ai-other",
        "apple-intelligence",
        "mistral",
        "deepseek",
        "character",
        "perplexity",
        "groq",
        "anthropic",
    ];
    ai_keywords.iter().any(|&k| name.contains(k))
}
