use anyhow::{Context, Result};
use ipnet::{Ipv4Net, Ipv6Net};
use ipnetwork::IpNetwork;
use maxminddb::Reader;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Deserialize)]
struct CountryRecord {
    country: Option<Country>,
}

#[derive(Deserialize)]
struct Country {
    iso_code: Option<String>,
}

#[derive(Deserialize)]
struct AsnRecord {
    autonomous_system_number: Option<u32>,
    autonomous_system_organization: Option<String>,
}

#[derive(Deserialize)]
struct UpstreamSource {
    name: String,
    url: String,
    category: String,
    target: String,
    enabled: bool,
}

#[derive(Deserialize)]
struct SingBoxRuleSet {
    #[serde(default)]
    rules: Vec<SingBoxRule>,
}

#[derive(Deserialize, Default)]
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
];

struct AsnTarget {
    name: &'static str,
    asns: &'static [u32],
}

#[derive(Deserialize)]
struct AsnMap {
    services: HashMap<String, AsnService>,
}

#[derive(Deserialize)]
struct AsnService {
    asns: Vec<u32>,
}

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

const ASN_TARGETS: &[AsnTarget] = &[
    AsnTarget {
        name: "asn-cloudflare",
        asns: &[13335, 209242, 395747, 133877],
    },
    AsnTarget {
        name: "asn-google",
        asns: &[15169, 396982, 36040, 43515, 16591, 19527],
    },
    AsnTarget {
        name: "asn-microsoft",
        asns: &[8075, 8068, 8069, 8070, 8071, 8072, 8073, 8074],
    },
    AsnTarget {
        name: "asn-amazon",
        asns: &[16509, 14618, 8987, 7224],
    },
    AsnTarget {
        name: "asn-facebook",
        asns: &[32934, 54115, 63293],
    },
    AsnTarget {
        name: "asn-telegram",
        asns: &[62041, 62014, 59930, 44907, 211157],
    },
    AsnTarget {
        name: "asn-netflix",
        asns: &[2906, 40027, 55095],
    },
    AsnTarget {
        name: "asn-github",
        asns: &[36459, 54113],
    },
    AsnTarget {
        name: "asn-twitter",
        asns: &[13414, 35995],
    },
    AsnTarget {
        name: "asn-apple",
        asns: &[714, 6185, 2709],
    },
    AsnTarget {
        name: "asn-discord",
        asns: &[393577, 836785],
    },
    AsnTarget {
        name: "asn-spotify",
        asns: &[8403],
    },
    AsnTarget {
        name: "asn-steam",
        asns: &[32590, 17012],
    },
    AsnTarget {
        name: "asn-disney",
        asns: &[19679],
    },
    AsnTarget {
        name: "asn-oracle",
        asns: &[31898],
    },
    AsnTarget {
        name: "asn-akamai",
        asns: &[16625, 20940, 3131, 33905, 34164, 34850, 43639, 53235, 54104],
    },
    AsnTarget {
        name: "asn-twitch",
        asns: &[46489],
    },
    AsnTarget {
        name: "asn-alibaba",
        asns: &[37963, 45102, 132335],
    },
    AsnTarget {
        name: "asn-tencent",
        asns: &[132203, 132591, 133478, 133543],
    },
    AsnTarget {
        name: "asn-bytedance",
        asns: &[138690],
    },
    AsnTarget {
        name: "asn-baidu",
        asns: &[55967, 134177],
    },
];

#[tokio::main]
async fn main() -> Result<()> {
    let root = find_root();
    let command = std::env::args().nth(1).unwrap_or_else(|| "all".to_string());
    let write = std::env::args().any(|arg| arg == "--write");

    match command.as_str() {
        "all" => {
            sync_upstream_rules(&root).await?;
            normalize_sources(&root, write)?;
            extract_geoip_asn(&root).await?;
        }
        "sync" | "sync-upstream" => sync_upstream_rules(&root).await?,
        "geo" | "geoip" | "asn" => extract_geoip_asn(&root).await?,
        "normalize" | "normalise" => normalize_sources(&root, write)?,
        "help" | "--help" | "-h" => print_help(),
        other => {
            anyhow::bail!("unknown rulekit command: {other}");
        }
    }

    Ok(())
}

fn print_help() {
    println!("Gins-Rules rulekit");
    println!();
    println!("Usage:");
    println!("  rulekit all [--write]        Sync, check source rules and extract GeoIP/ASN");
    println!("  rulekit sync                 Sync upstream rules and QX/Loon parsers");
    println!("  rulekit normalize [--write]  Normalize source rule text files");
    println!("  rulekit geo                  Extract GeoIP/ASN CIDR outputs");
}

async fn sync_upstream_rules(root: &Path) -> Result<()> {
    println!("============================================================");
    println!("  Gins-Rules Upstream Syncer (Rust)");
    println!("============================================================");

    let config_path = root.join("source").join("sources.json");
    let config_data = fs::read(&config_path)
        .with_context(|| format!("failed to read {}", config_path.display()))?;
    let sources: Vec<UpstreamSource> = serde_json::from_slice(&config_data)
        .with_context(|| format!("failed to parse {}", config_path.display()))?;

    let client = http_client()?;
    let mut merged: BTreeMap<String, BTreeMap<String, BTreeSet<String>>> = BTreeMap::new();

    for source in sources {
        if !source.enabled {
            println!("  [SKIP] {}", source.name);
            continue;
        }

        println!(
            "  [FETCH] {} -> {}/{}",
            source.name, source.category, source.target
        );
        match fetch_text(&client, &source.url).await {
            Ok(content) => {
                let rules = process_upstream_rules(&content);
                let target_rules = merged
                    .entry(source.category)
                    .or_default()
                    .entry(source.target)
                    .or_default();
                target_rules.extend(rules);
            }
            Err(err) => {
                println!("  [WARN] {}: {err:#}", source.url);
            }
        }
    }

    let upstream_root = root.join("source").join("upstream");
    for (category, targets) in merged {
        let out_dir = upstream_root.join(category);
        fs::create_dir_all(&out_dir)?;
        for (target, rules) in targets {
            let out_path = out_dir.join(format!("{target}.txt"));
            let body = if rules.is_empty() {
                String::new()
            } else {
                format!("{}\n", rules.into_iter().collect::<Vec<_>>().join("\n"))
            };
            fs::write(&out_path, body)?;
            println!("  [WRITE] {}", out_path.strip_prefix(root)?.display());
        }
    }

    sync_resource_parsers(root, &client).await?;

    println!("============================================================");
    Ok(())
}

async fn sync_resource_parsers(root: &Path, client: &reqwest::Client) -> Result<()> {
    let qx_url =
        "https://raw.githubusercontent.com/KOP-XIAO/QuantumultX/master/Scripts/resource-parser.js";
    let loon_url = "https://github.com/sub-store-org/Sub-Store/releases/latest/download/sub-store-parser.loon.min.js";

    println!("  [FETCH] QX Resource Parser");
    let qx = fetch_text(client, qx_url).await?;
    fs::write(
        root.join("source").join("QX-Resource-Parser.js"),
        clean_qx_parser(&qx),
    )?;

    println!("  [FETCH] Loon Resource Parser");
    let loon = fetch_text(client, loon_url).await?;
    fs::write(root.join("source").join("Loon-Resource-Parser.js"), loon)?;

    Ok(())
}

fn process_upstream_rules(content: &str) -> BTreeSet<String> {
    if let Ok(rule_set) = serde_json::from_str::<SingBoxRuleSet>(content) {
        if !rule_set.rules.is_empty() {
            let mut out = BTreeSet::new();
            for rule in rule_set.rules {
                out.extend(rule.domain_suffix.into_iter().map(clean_domain_suffix));
                out.extend(
                    rule.domain
                        .into_iter()
                        .map(|v| format!("full:{}", clean_value(&v))),
                );
                out.extend(
                    rule.domain_keyword
                        .into_iter()
                        .map(|v| format!("keyword:{}", clean_value(&v))),
                );
                out.extend(
                    rule.domain_regex
                        .into_iter()
                        .map(|v| format!("regexp:{}", clean_value(&v))),
                );
                out.extend(rule.ip_cidr.into_iter().map(|v| clean_value(&v)));
                out.extend(
                    rule.process_name
                        .into_iter()
                        .map(|v| format!("process:{}", clean_value(&v))),
                );
                out.extend(
                    rule.user_agent
                        .into_iter()
                        .map(|v| format!("user-agent:{}", clean_value(&v))),
                );
            }
            out.retain(|rule| !rule.is_empty());
            return out;
        }
    }

    let mut out = BTreeSet::new();
    for raw in content.lines() {
        let mut line = raw.trim();
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with(';')
            || line.starts_with("//")
            || line == "payload:"
        {
            continue;
        }
        line = line.trim_start_matches('-').trim();
        if line.starts_with('\'') && line.ends_with('\'') && line.len() > 1 {
            line = &line[1..line.len() - 1];
        }
        if line.starts_with('"') && line.ends_with('"') && line.len() > 1 {
            line = &line[1..line.len() - 1];
        }

        let parts: Vec<&str> = line.split(',').map(str::trim).collect();
        if parts.len() >= 2 {
            let value = clean_value(parts[1]);
            let rule = match parts[0].to_ascii_uppercase().as_str() {
                "DOMAIN-SUFFIX" | "HOST-SUFFIX" => clean_domain_suffix(&value),
                "DOMAIN" | "HOST" => format!("full:{value}"),
                "DOMAIN-KEYWORD" | "HOST-KEYWORD" => format!("keyword:{value}"),
                "DOMAIN-REGEX" | "URL-REGEX" => format!("regexp:{value}"),
                "PROCESS-NAME" | "PROCESS" => format!("process:{value}"),
                "USER-AGENT" => format!("user-agent:{value}"),
                "IP-CIDR" | "IP-CIDR6" | "IP6-CIDR" | "SRC-IP-CIDR" => value,
                "IP-ASN" => format!("asn:{value}"),
                _ => String::new(),
            };
            if !rule.is_empty() {
                out.insert(rule);
            }
        } else if !line.contains(',') {
            out.insert(normalize_external_rule_line(line));
        }
    }
    out.retain(|rule| !rule.is_empty());
    out
}

fn normalize_external_rule_line(line: &str) -> String {
    let line = normalize_rule_line(line);
    match line.split_once(':') {
        Some(("domain", value)) => clean_domain_suffix(value),
        Some(("full", value)) => format!("full:{}", clean_value(value)),
        Some(("keyword", value)) => format!("keyword:{}", clean_value(value)),
        Some(("regexp", value)) | Some(("regex", value)) => {
            format!("regexp:{}", clean_value(value))
        }
        Some(("process", value)) => format!("process:{}", clean_value(value)),
        Some(("user-agent", value)) => format!("user-agent:{}", clean_value(value)),
        Some(("asn", value)) => format!("asn:{}", clean_value(value)),
        _ => line,
    }
}

fn clean_qx_parser(content: &str) -> String {
    let Some(start) = content.find("/**") else {
        return content.to_string();
    };
    let Some(relative_end) = content[start..].find("*/") else {
        return content.to_string();
    };
    let end = start + relative_end + 2;
    let header =
        "/** \n * Gins-Rules QX Resource Parser\n * - Automated Proxy Rule Conversion\n */";
    format!("{header}\n{}", &content[end..])
}

fn clean_value(value: &str) -> String {
    let mut value = value.trim().to_string();
    if let Some((left, _)) = value.split_once("//") {
        value = left.trim().to_string();
    }
    value
        .trim_end_matches(",no-resolve")
        .trim()
        .trim_matches('\'')
        .trim_matches('"')
        .to_string()
}

fn clean_domain_suffix(value: impl AsRef<str>) -> String {
    let value = clean_value(value.as_ref());
    value
        .trim_start_matches("+.")
        .trim_start_matches('.')
        .to_string()
}

fn http_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Gins-Rules/1.0 (https://github.com/IchimaruGin728/Gins-Rules)")
        .build()
        .context("failed to build HTTP client")
}

async fn fetch_text(client: &reqwest::Client, url: &str) -> Result<String> {
    let response = client.get(url).send().await?;
    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("{url}: HTTP {status}");
    }
    Ok(response.text().await?)
}

async fn extract_geoip_asn(root: &Path) -> Result<()> {
    let ip_dir = root.join("source").join("upstream").join("ip");
    let compiled_dir = root.join("compiled");
    let tmp_dir = root.join(".mmdb-cache");

    fs::create_dir_all(&ip_dir)?;
    fs::create_dir_all(&compiled_dir)?;
    fs::create_dir_all(&tmp_dir)?;

    println!("============================================================");
    println!("  Gins-Rules MMDB Parser (Rust Extreme Version)");
    println!("============================================================");

    let mut country_cidrs: HashMap<String, HashSet<IpNetwork>> = HashMap::new();
    let mut asn_cidrs: HashMap<u32, Vec<AsnNetwork>> = HashMap::new();

    for src in MMDB_SOURCES {
        println!("\n  Processing {}...", src.name);
        let local_path = tmp_dir.join(format!("{}.mmdb", src.name));

        if !local_path.exists() {
            println!("  Downloading {}...", src.url);
            download_file(src.url, &local_path).await?;
        }

        let reader = Reader::open_readfile(&local_path)?;
        match src.r#type {
            "country" => extract_country_cidrs(&reader, &mut country_cidrs)?,
            "asn" => extract_asn_cidrs(&reader, &mut asn_cidrs)?,
            _ => 0,
        };
    }

    let common_regions: HashSet<&str> = vec!["CN", "SG", "TW", "JP", "US"].into_iter().collect();
    let mut not_cn = HashSet::new();

    for (code, nets) in &country_cidrs {
        if code != "CN" {
            for net in nets {
                not_cn.insert(*net);
            }
        }
        if common_regions.contains(code.as_str()) {
            let out_path = ip_dir.join(format!("{}.txt", code.to_lowercase()));
            write_aggregated_ip_list(&out_path, nets)?;
            println!(
                "  [WRITE] ip/{}.txt: {} networks",
                code.to_lowercase(),
                nets.len()
            );
        }
    }

    if !not_cn.is_empty() {
        write_aggregated_ip_list(&ip_dir.join("!cn.txt"), &not_cn)?;
        println!(
            "  [WRITE] ip/!cn.txt: {} networks (All non-CN)",
            not_cn.len()
        );
    }

    let targets = discover_asn_targets(&root)?;
    let mut prefix_index: BTreeMap<String, Vec<AsnPrefixRecord>> = BTreeMap::new();

    for (target_name, target_asns) in targets {
        let mut merged = HashSet::new();
        let mut records = Vec::new();

        for asn in target_asns {
            let Some(nets) = asn_cidrs.get(&asn) else {
                continue;
            };

            let mut per_asn = HashSet::new();
            let mut org: Option<String> = None;
            for entry in nets {
                merged.insert(entry.net);
                per_asn.insert(entry.net);
                if org.is_none() {
                    org = entry.org.clone();
                }
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
            println!(
                "  [WRITE] ip/{}.txt: {} networks",
                target_name,
                merged.len()
            );
            prefix_index.insert(target_name, records);
        }
    }

    let index_path = compiled_dir.join("asn-prefix-index.json");
    let index_json = serde_json::to_vec_pretty(&prefix_index)?;
    fs::write(&index_path, index_json)?;
    println!(
        "  [WRITE] compiled/asn-prefix-index.json: {} targets",
        prefix_index.len()
    );

    println!("============================================================");
    Ok(())
}

fn normalize_sources(root: &Path, write: bool) -> Result<()> {
    println!("============================================================");
    println!("  Gins-Rules Rule Normalizer (Rust)");
    println!("============================================================");

    let source_root = root.join("source");
    let dirs = [
        source_root.join("proxy"),
        source_root.join("direct"),
        source_root.join("reject"),
        source_root.join("ip"),
        source_root.join("upstream").join("proxy"),
        source_root.join("upstream").join("direct"),
        source_root.join("upstream").join("reject"),
        source_root.join("upstream").join("ip"),
    ];

    let mut files = 0usize;
    let mut changed = 0usize;
    let mut rules = 0usize;

    for dir in dirs {
        if !dir.exists() {
            continue;
        }
        for path in list_txt_paths(&dir)? {
            files += 1;
            let before = fs::read_to_string(&path)?;
            let normalized = normalize_rule_text(&before);
            rules += normalized
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count();
            if before != normalized {
                changed += 1;
                if write {
                    fs::write(&path, normalized)?;
                }
            }
        }
    }

    println!("  [NORMALIZE] files: {files}");
    println!("  [NORMALIZE] changed: {changed}");
    println!(
        "  [NORMALIZE] mode: {}",
        if write { "write" } else { "check" }
    );
    println!("  [NORMALIZE] rules: {rules}");
    println!("============================================================");
    Ok(())
}

fn list_txt_paths(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("txt") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn normalize_rule_text(input: &str) -> String {
    let mut comments = BTreeSet::new();
    let mut rules = BTreeSet::new();

    for raw in input.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') {
            comments.insert(line.to_string());
            continue;
        }

        let normalized = normalize_rule_line(line);
        if !normalized.is_empty() {
            rules.insert(normalized);
        }
    }

    let mut output = Vec::new();
    output.extend(comments);
    output.extend(rules);
    if output.is_empty() {
        String::new()
    } else {
        format!("{}\n", output.into_iter().collect::<Vec<_>>().join("\n"))
    }
}

fn normalize_rule_line(line: &str) -> String {
    let mut value = line.trim().trim_end_matches('\r').trim().to_string();
    if let Some((left, _comment)) = value.split_once(" #") {
        value = left.trim().to_string();
    }
    if value.starts_with("+.") {
        value = value.trim_start_matches("+.").to_string();
    }
    if value.starts_with('.') && value.chars().filter(|c| *c == '.').count() > 1 {
        value = value.trim_start_matches('.').to_string();
    }
    if value.starts_with("DOMAIN-SUFFIX,") {
        value = value.trim_start_matches("DOMAIN-SUFFIX,").to_string();
    }
    value
}

fn extract_country_cidrs(
    reader: &Reader<Vec<u8>>,
    storage: &mut HashMap<String, HashSet<IpNetwork>>,
) -> Result<usize> {
    let mut count = 0;
    let iter = reader.networks(Default::default())?;
    for result in iter {
        let lookup = result?;
        if let Some(code) = lookup
            .decode::<CountryRecord>()?
            .and_then(|r| r.country)
            .and_then(|c| c.iso_code)
        {
            storage.entry(code).or_default().insert(lookup.network()?);
            count += 1;
        }
    }
    Ok(count)
}

fn extract_asn_cidrs(
    reader: &Reader<Vec<u8>>,
    storage: &mut HashMap<u32, Vec<AsnNetwork>>,
) -> Result<usize> {
    let mut count = 0;
    let iter = reader.networks(Default::default())?;
    for result in iter {
        let lookup = result?;
        if let Some(record) = lookup.decode::<AsnRecord>()? {
            if let Some(asn) = record.autonomous_system_number {
                storage.entry(asn).or_default().push(AsnNetwork {
                    net: lookup.network()?,
                    org: record.autonomous_system_organization,
                });
            }
            count += 1;
        }
    }
    Ok(count)
}

async fn download_file(url: &str, dest: &Path) -> Result<()> {
    let response = reqwest::get(url).await?;
    let content = response.bytes().await?;
    fs::write(dest, content)?;
    Ok(())
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

fn discover_asn_targets(root: &Path) -> Result<BTreeMap<String, BTreeSet<u32>>> {
    let mut targets: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();

    for target in ASN_TARGETS {
        let entry = targets.entry(target.name.to_string()).or_default();
        for &asn in target.asns {
            entry.insert(asn);
        }
    }

    let asn_map_path = root.join("source").join("asn-map.json");
    if let Ok(data) = fs::read(&asn_map_path) {
        if let Ok(asn_map) = serde_json::from_slice::<AsnMap>(&data) {
            for (service, def) in asn_map.services {
                let entry = targets.entry(format!("asn-{}", service)).or_default();
                for asn in def.asns {
                    entry.insert(asn);
                }
            }
        }
    }

    for dir in [
        root.join("source").join("ip"),
        root.join("source").join("upstream").join("ip"),
    ] {
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
            if !name.starts_with("asn-") {
                continue;
            }
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

fn find_root() -> PathBuf {
    let mut curr = std::env::current_dir().unwrap();
    while curr.parent().is_some() {
        if curr.join(".git").exists() {
            return curr;
        }
        curr = curr.parent().unwrap().to_path_buf();
    }
    curr
}
