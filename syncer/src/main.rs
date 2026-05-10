use anyhow::Result;
use clap::{Parser, Subcommand};
use hex;
use ipnet::{Ipv4Net, Ipv6Net};
use ipnetwork::IpNetwork;
use maxminddb::Reader;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Sync,
    Icons,
    He,
    Geo,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpstreamSource {
    name: String,
    url: String,
    category: String,
    target: String,
    enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct SingBoxRuleSet {
    #[serde(default)]
    rules: Vec<SingBoxRule>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct SingBoxRule {
    #[serde(default)]
    domain: Vec<String>,
    #[serde(default)]
    domain_suffix: Vec<String>,
    #[serde(default)]
    domain_keyword: Vec<String>,
    #[serde(default)]
    domain_regex: Vec<String>,
    #[serde(default)]
    ip_cidr: Vec<String>,
    #[serde(default)]
    process_name: Vec<String>,
    #[serde(default)]
    user_agent: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct IconSource {
    name: String,
    url: String,
    theme: String,
    enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NormalizedIcon {
    name: String,
    url: String,
    source: String,
    theme: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RawIcon {
    #[serde(default)]
    name: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    tag: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ASNMap {
    services: HashMap<String, ServiceDef>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServiceDef {
    asns: Vec<u32>,
    org: String,
}

#[derive(Debug, Deserialize)]
struct RIPEResponse {
    data: RIPEData,
    status: String,
}

#[derive(Debug, Deserialize)]
struct RIPEData {
    prefixes: Vec<RIPEPrefix>,
}

#[derive(Debug, Deserialize)]
struct RIPEPrefix {
    prefix: String,
}

#[derive(Deserialize)]
struct CountryRecord {
    country: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct AsnRecord {
    #[serde(alias = "asn", alias = "autonomous_system_number")]
    autonomous_system_number: Option<u32>,
    #[serde(
        alias = "name",
        alias = "asn_org",
        alias = "organization",
        alias = "autonomous_system_organization"
    )]
    autonomous_system_organization: Option<String>,
}

struct MmdbSource {
    name: &'static str,
    url: &'static str,
    r#type: &'static str,
}

const MMDB_SOURCES: &[MmdbSource] = &[
    MmdbSource {
        name: "ipinfo.country",
        url: "https://github.com/xream/geoip/releases/latest/download/ipinfo.country.mmdb",
        r#type: "country",
    },
    MmdbSource {
        name: "ip2location.country",
        url: "https://github.com/xream/geoip/releases/latest/download/ip2location.country.mmdb",
        r#type: "country",
    },
    MmdbSource {
        name: "ipinfo.asn",
        url: "https://github.com/xream/geoip/releases/latest/download/ipinfo.asn.mmdb",
        r#type: "asn",
    },
    MmdbSource {
        name: "ip2location.asn",
        url: "https://github.com/xream/geoip/releases/latest/download/ip2location.asn.mmdb",
        r#type: "asn",
    },
    MmdbSource {
        name: "loyalsoldier.country",
        url:
            "https://raw.githubusercontent.com/Loyalsoldier/geoip/release/Country-without-asn.mmdb",
        r#type: "country",
    },
    MmdbSource {
        name: "loyalsoldier.asn",
        url: "https://raw.githubusercontent.com/Loyalsoldier/geoip/release/GeoLite2-ASN.mmdb",
        r#type: "asn",
    },
];

#[derive(Clone)]
struct AsnNetwork {
    net: IpNetwork,
    org: Option<String>,
}

#[derive(Serialize)]
struct AsnPrefixRecord {
    asn: u32,
    cidr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    org: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = find_root();
    match cli.command {
        Commands::Sync => run_sync(&root).await?,
        Commands::Icons => run_icons(&root).await?,
        Commands::He => run_he(&root).await?,
        Commands::Geo => run_geo(&root).await?,
    }
    Ok(())
}

async fn run_sync(root: &Path) -> Result<()> {
    let config_path = root.join("source").join("sources.json");
    let upstream_dir = root.join("source").join("upstream");
    let config_data = fs::read_to_string(&config_path)?;
    let sources: Vec<UpstreamSource> = serde_json::from_str(&config_data)?;
    let mut merged_results: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Gins-Rules/1.0")
        .build()?;
    for src in sources {
        if !src.enabled {
            continue;
        }
        println!("  Fetching {} (targeting {})...", src.name, src.target);
        if let Ok(resp) = client.get(&src.url).send().await {
            if resp.status().is_success() {
                let content = resp.text().await?;
                let rules = process_rules(&content);
                merged_results
                    .entry(src.category)
                    .or_default()
                    .entry(src.target)
                    .or_default()
                    .extend(rules);
            }
        }
    }
    for (cat, targets) in merged_results {
        let out_dir = upstream_dir.join(cat);
        fs::create_dir_all(&out_dir)?;
        for (name, rules) in targets {
            fs::write(
                out_dir.join(format!("{}.txt", name)),
                rules.join("\n") + "\n",
            )?;
            println!("  [SUCCESS] Written {} rules to {}.txt", rules.len(), name);
        }
    }
    sync_qx_parser(&client, root).await?;
    sync_loon_parser(&client, root).await?;
    Ok(())
}

async fn run_icons(root: &Path) -> Result<()> {
    let config_path = root.join("source").join("icons.json");
    let out_path = root.join("compiled").join("Gins-Icons.json");
    let hash_path = root.join("source").join("icons-hash.json");
    let dashboard_path = root
        .join("dashboard")
        .join("public")
        .join("icons-catalog.json");
    let config_data = fs::read_to_string(&config_path)?;
    let sources: Vec<IconSource> = serde_json::from_str(&config_data)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Gins-Rules/1.0")
        .build()?;
    let all_icons = Arc::new(Mutex::new(Vec::new()));
    let mut futures = Vec::new();
    for src in sources {
        if !src.enabled {
            continue;
        }
        let client_clone = client.clone();
        let all_icons_clone = Arc::clone(&all_icons);
        futures.push(async move {
            if let Ok(resp) = client_clone.get(&src.url).send().await {
                if let Ok(body) = resp.text().await {
                    let raw_icons = extract_icons(&body);
                    let mut normalized = Vec::new();
                    let mut seen = HashSet::new();
                    for icon in raw_icons {
                        if icon.url.is_empty() || seen.contains(&icon.url) {
                            continue;
                        }
                        seen.insert(icon.url.clone());
                        let mut name = if !icon.name.is_empty() {
                            icon.name
                        } else {
                            icon.tag
                        };
                        if name.is_empty() {
                            name = icon.url.split('/').last().unwrap_or("icon").to_string();
                        }
                        normalized.push(NormalizedIcon {
                            name: name.trim().to_string(),
                            url: icon.url,
                            source: src.name.clone(),
                            theme: src.theme.clone(),
                        });
                    }
                    all_icons_clone.lock().unwrap().extend(normalized);
                }
            }
        });
    }
    futures::future::join_all(futures).await;
    let mut icons = all_icons.lock().unwrap().clone();
    icons.sort_by(|a, b| {
        if a.source != b.source {
            a.source.cmp(&b.source)
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });
    let final_data = serde_json::to_string_pretty(&icons)?;
    fs::create_dir_all(out_path.parent().unwrap())?;
    fs::write(&out_path, &final_data)?;
    fs::create_dir_all(dashboard_path.parent().unwrap())?;
    fs::write(&dashboard_path, &final_data)?;
    let mut hasher = Sha256::new();
    hasher.update(&final_data);
    let hash_str = hex::encode(hasher.finalize());
    let hash_json = serde_json::json!({ "sha256": hash_str, "total": icons.len().to_string() });
    fs::write(hash_path, serde_json::to_string_pretty(&hash_json)?)?;
    Ok(())
}

async fn run_he(root: &Path) -> Result<()> {
    let asn_map_path = root.join("source").join("asn-map.json");
    let out_dir = root.join("source").join("upstream").join("ip");
    fs::create_dir_all(&out_dir)?;
    let asn_map_data = fs::read_to_string(&asn_map_path)?;
    let asn_map: ASNMap = serde_json::from_str(&asn_map_data)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Gins-Rules/1.0")
        .build()?;
    let mut results: HashMap<String, HashSet<String>> = HashMap::new();
    for (svc_name, svc_def) in &asn_map.services {
        results.insert(svc_name.clone(), HashSet::new());
        for &asn in &svc_def.asns {
            let url = format!(
                "https://stat.ripe.net/data/announced-prefixes/data.json?resource=AS{}",
                asn
            );
            if let Ok(resp) = client.get(&url).send().await {
                if let Ok(ripe_resp) = resp.json::<RIPEResponse>().await {
                    if ripe_resp.status == "ok" {
                        for p in ripe_resp.data.prefixes {
                            results.get_mut(svc_name).unwrap().insert(p.prefix);
                        }
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
    let official_sources = vec![
        ("telegram", "https://core.telegram.org/resources/cidr.txt"),
        ("cloudflare", "https://www.cloudflare.com/ips-v4"),
        ("cloudflare", "https://www.cloudflare.com/ips-v6"),
    ];
    for (svc, url) in official_sources {
        if let Ok(resp) = client.get(url).send().await {
            if let Ok(body) = resp.text().await {
                for line in body.lines() {
                    let line = line.trim();
                    if line.contains('/') && !line.starts_with('#') {
                        results
                            .entry(svc.to_string())
                            .or_default()
                            .insert(line.to_string());
                    }
                }
            }
        }
    }
    for (svc_name, cidrs) in results {
        if cidrs.is_empty() {
            continue;
        }
        let mut sorted: Vec<_> = cidrs.into_iter().collect();
        sorted.sort();
        fs::write(
            out_dir.join(format!("asn-{}.txt", svc_name)),
            sorted.join("\n") + "\n",
        )?;
        println!("  [OK] asn-{}.txt → {} CIDRs", svc_name, sorted.len());
    }
    Ok(())
}

async fn run_geo(root: &Path) -> Result<()> {
    let ip_dir = root.join("source").join("upstream").join("ip");
    let compiled_dir = root.join("compiled");
    let tmp_dir = root.join(".mmdb-cache");
    fs::create_dir_all(&ip_dir)?;
    fs::create_dir_all(&compiled_dir)?;
    fs::create_dir_all(&tmp_dir)?;
    let mut country_cidrs: HashMap<String, HashSet<IpNetwork>> = HashMap::new();
    let mut asn_cidrs: HashMap<u32, Vec<AsnNetwork>> = HashMap::new();
    for src in MMDB_SOURCES {
        let local_path = tmp_dir.join(format!("{}.mmdb", src.name));
        if !local_path.exists() {
            println!("  Downloading {}...", src.url);
            let response = reqwest::get(src.url).await?.bytes().await?;
            fs::write(&local_path, response)?;
        }
        let reader = Reader::open_readfile(&local_path)?;
        match src.r#type {
            "country" => {
                let iter = reader.networks(Default::default())?;
                for result in iter {
                    let lookup = result?;
                    if let Ok(Some(record)) = lookup.decode::<CountryRecord>() {
                        if let Some(val) = record.country {
                            let code = if val.is_string() {
                                val.as_str().map(|s| s.to_string())
                            } else {
                                val.get("iso_code")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string())
                            };
                            if let Some(c) = code {
                                country_cidrs
                                    .entry(c.to_uppercase())
                                    .or_default()
                                    .insert(lookup.network()?);
                            }
                        }
                    }
                }
            }
            "asn" => {
                let iter = reader.networks(Default::default())?;
                for result in iter {
                    let lookup = result?;
                    if let Ok(Some(record)) = lookup.decode::<AsnRecord>() {
                        if let Some(asn) = record.autonomous_system_number {
                            asn_cidrs.entry(asn).or_default().push(AsnNetwork {
                                net: lookup.network()?,
                                org: record.autonomous_system_organization.clone(),
                            });
                        }
                    }
                }
            }
            _ => (),
        }
    }
    let mut not_cn = HashSet::new();
    let common_regions: HashSet<&str> = vec!["CN", "SG", "TW", "JP", "HK", "UK"] // Removed US, KR, DE, FR
        .into_iter()
        .collect();
    for (code, nets) in &country_cidrs {
        if code != "CN" {
            for net in nets {
                not_cn.insert(*net);
            }
        }
        if common_regions.contains(code.as_str()) {
            write_aggregated_ip_list(&ip_dir.join(format!("{}.txt", code.to_lowercase())), nets)?;
        }
    }
    if !not_cn.is_empty() {
        write_aggregated_ip_list(&ip_dir.join("!cn.txt"), &not_cn)?;
    }

    // Export ALL country CIDRs to a single index for the compiler
    let mut full_country_index: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (code, nets) in &country_cidrs {
        full_country_index.insert(code.clone(), aggregate_networks(nets));
    }
    println!(
        "  [Geo] Exporting {} countries to full-country-index.json",
        full_country_index.len()
    );
    fs::write(
        compiled_dir.join("full-country-index.json"),
        serde_json::to_vec_pretty(&full_country_index)?,
    )?;

    let targets = discover_asn_targets(root)?;
    let mut prefix_index: BTreeMap<String, Vec<AsnPrefixRecord>> = BTreeMap::new();

    // Track ALL ASNs with their names and CIDRs
    let mut all_asns_full: BTreeMap<u32, (Option<String>, Vec<IpNetwork>)> = BTreeMap::new();
    for (asn, networks) in &asn_cidrs {
        let org = networks.first().and_then(|n| n.org.clone());
        let nets: Vec<IpNetwork> = networks.iter().map(|n| n.net).collect();
        all_asns_full.insert(*asn, (org, nets));
    }

    for (target_name, target_asns) in targets {
        let mut merged = HashSet::new();
        let mut records = Vec::new();
        for asn in target_asns {
            let Some((org, nets)) = all_asns_full.get(&asn) else {
                continue;
            };
            let mut per_asn = HashSet::new();
            for net in nets {
                merged.insert(*net);
                per_asn.insert(*net);
            }
            for cidr in aggregate_networks(&per_asn) {
                records.push(AsnPrefixRecord {
                    asn,
                    cidr,
                    org: org.clone(),
                });
            }
        }
        if !merged.is_empty() {
            write_aggregated_ip_list(&ip_dir.join(format!("{}.txt", target_name)), &merged)?;
            records.sort_by(|a, b| a.asn.cmp(&b.asn).then_with(|| a.cidr.cmp(&b.cidr)));
            prefix_index.insert(target_name, records);
        }
    }
    let index_json = serde_json::to_vec_pretty(&prefix_index)?;
    fs::write(compiled_dir.join("asn-prefix-index.json"), index_json)?;

    // Export ALL ASNs to a full index for geoasn.mmdb
    let mut full_asn_index: Vec<AsnPrefixRecord> = Vec::new();
    for (asn, (org, nets)) in all_asns_full {
        let net_set: HashSet<IpNetwork> = nets.into_iter().collect();
        for cidr in aggregate_networks(&net_set) {
            full_asn_index.push(AsnPrefixRecord {
                asn,
                cidr,
                org: org.clone(),
            });
        }
    }
    // Sort for deterministic build and MMDB efficiency
    full_asn_index.sort_by(|a, b| a.cidr.cmp(&b.cidr));

    println!(
        "  [Geo] Exporting {} ASN CIDR records to full-asn-index.json",
        full_asn_index.len()
    );
    fs::write(
        compiled_dir.join("full-asn-index.json"),
        serde_json::to_vec_pretty(&full_asn_index)?,
    )?;

    Ok(())
}
fn discover_asn_targets(root: &Path) -> Result<BTreeMap<String, BTreeSet<u32>>> {
    let mut targets: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();
    let asn_map_path = root.join("source").join("asn-map.json");
    if let Ok(data) = fs::read(&asn_map_path) {
        if let Ok(asn_map) = serde_json::from_slice::<ASNMap>(&data) {
            for (service, def) in asn_map.services {
                let entry = targets.entry(format!("asn-{}", service)).or_default();
                for asn in def.asns {
                    entry.insert(asn);
                }
            }
        }
    }
    let dirs = [
        root.join("source").join("proxy"),
        root.join("source").join("direct"),
        root.join("source").join("reject"),
        root.join("source").join("ip"),
        root.join("source").join("upstream").join("proxy"),
        root.join("source").join("upstream").join("direct"),
        root.join("source").join("upstream").join("reject"),
        root.join("source").join("upstream").join("ip"),
    ];
    for dir in dirs {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("txt") {
                continue;
            }
            let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let data = fs::read_to_string(&path)?;
            for line in data.lines() {
                let line = line.trim();
                let asn = line
                    .strip_prefix("asn:")
                    .or_else(|| line.strip_prefix("AS"))
                    .and_then(|value| value.trim().parse::<u32>().ok());
                if let Some(asn) = asn {
                    targets.entry(name.to_string()).or_default().insert(asn);
                }
            }
        }
    }
    Ok(targets)
}

fn write_aggregated_ip_list(path: &Path, nets: &HashSet<IpNetwork>) -> Result<()> {
    let aggregated = aggregate_networks(nets);
    let file = fs::File::create(path)?;
    let mut writer = BufWriter::new(file);
    for n in aggregated {
        writeln!(writer, "{}", n)?;
    }
    writer.flush()?;
    Ok(())
}

fn aggregate_networks(nets: &HashSet<IpNetwork>) -> Vec<String> {
    let mut v4: Vec<Ipv4Net> = Vec::new();
    let mut v6: Vec<Ipv6Net> = Vec::new();
    for net in nets {
        match net {
            IpNetwork::V4(n) => {
                if let Ok(converted) = Ipv4Net::new(n.ip(), n.prefix()) {
                    v4.push(converted);
                }
            }
            IpNetwork::V6(n) => {
                if let Ok(converted) = Ipv6Net::new(n.ip(), n.prefix()) {
                    v6.push(converted);
                }
            }
        }
    }
    let mut out: Vec<String> = Vec::new();
    for n in Ipv4Net::aggregate(&v4) {
        out.push(n.to_string());
    }
    for n in Ipv6Net::aggregate(&v6) {
        out.push(n.to_string());
    }
    out.sort();
    out
}

fn process_rules(content: &str) -> Vec<String> {
    if let Ok(rs) = serde_json::from_str::<SingBoxRuleSet>(content) {
        let mut rules = Vec::new();
        for r in rs.rules {
            rules.extend(r.domain_suffix);
            for d in r.domain {
                rules.push(format!("full:{}", d));
            }
            for k in r.domain_keyword {
                rules.push(format!("keyword:{}", k));
            }
            for re in r.domain_regex {
                rules.push(format!("regexp:{}", re));
            }
            rules.extend(r.ip_cidr);
            for p in r.process_name {
                rules.push(format!("process:{}", p));
            }
            for u in r.user_agent {
                rules.push(format!("user-agent:{}", u));
            }
        }
        if !rules.is_empty() {
            return rules;
        }
    }
    content
        .lines()
        .filter_map(|l| {
            let l = l.trim();
            if l.is_empty() || l.starts_with('#') || l.starts_with(';') || l.starts_with("//") {
                return None;
            }
            let l = l
                .trim_start_matches('-')
                .trim()
                .trim_matches(|c| c == '"' || c == '\'');
            let parts: Vec<&str> = l.split(',').collect();
            if parts.len() >= 2 {
                let rt = parts[0].trim().to_uppercase();
                let v = parts[1]
                    .trim()
                    .trim_matches(|c| c == '"' || c == '\'')
                    .split("//")
                    .next()
                    .unwrap()
                    .trim()
                    .trim_end_matches(",no-resolve")
                    .to_string();
                match rt.as_str() {
                    "DOMAIN-SUFFIX" | "HOST-SUFFIX" => Some(v),
                    "DOMAIN" | "HOST" => Some(format!("full:{}", v)),
                    "DOMAIN-KEYWORD" | "HOST-KEYWORD" => Some(format!("keyword:{}", v)),
                    "DOMAIN-REGEX" | "URL-REGEX" => Some(format!("regexp:{}", v)),
                    "PROCESS-NAME" => Some(format!("process:{}", v)),
                    "USER-AGENT" => Some(format!("user-agent:{}", v)),
                    "IP-CIDR" | "IP-CIDR6" => Some(v),
                    "IP-ASN" => Some(format!("asn:{}", v)),
                    _ => None,
                }
            } else {
                let l = l.trim_start_matches('+').trim_start_matches('.').trim();
                if l.starts_with("domain:") {
                    return Some(l[7..].to_string());
                }
                if l.starts_with("full:")
                    || l.starts_with("keyword:")
                    || l.starts_with("regexp:")
                    || l.starts_with("process:")
                    || l.starts_with("user-agent:")
                    || l.starts_with("asn:")
                {
                    return Some(l.to_string());
                }
                if l.is_empty() {
                    return None;
                }
                Some(l.to_string())
            }
        })
        .collect()
}

fn extract_icons(data: &str) -> Vec<RawIcon> {
    if let Ok(list) = serde_json::from_str::<Vec<RawIcon>>(data) {
        return list;
    }
    if let Ok(m) = serde_json::from_str::<HashMap<String, serde_json::Value>>(data) {
        for k in ["icons", "items", "iconList", "list", "tubiao"] {
            if let Some(val) = m.get(k) {
                if let Ok(list) = serde_json::from_value::<Vec<RawIcon>>(val.clone()) {
                    return list;
                }
            }
        }
    }
    Vec::new()
}

async fn sync_qx_parser(client: &reqwest::Client, root: &Path) -> Result<()> {
    let content = client.get("https://raw.githubusercontent.com/KOP-XIAO/QuantumultX/master/Scripts/resource-parser.js").send().await?.text().await?;
    let cleaned = if let Some(end) = content.find("*/") {
        format!(
            "/** \n * Gins-Rules QX Resource Parser\n */\n{}",
            &content[end + 2..]
        )
    } else {
        content
    };
    fs::write(root.join("source").join("QX-Resource-Parser.js"), cleaned)?;
    Ok(())
}

async fn sync_loon_parser(client: &reqwest::Client, root: &Path) -> Result<()> {
    let content = client.get("https://github.com/sub-store-org/Sub-Store/releases/latest/download/sub-store-parser.loon.min.js").send().await?.text().await?;
    fs::write(root.join("source").join("Loon-Resource-Parser.js"), content)?;
    Ok(())
}

fn find_root() -> PathBuf {
    let mut d = std::env::current_dir().unwrap();
    while d.parent().is_some() {
        if d.join("source").join("sources.json").exists() {
            return d;
        }
        d.pop();
    }
    PathBuf::from(".")
}
