use anyhow::Result;
use ipnet::{Ipv4Net, Ipv6Net};
use ipnetwork::IpNetwork;
use maxminddb::Reader;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

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
            normalize_sources(&root, write)?;
            extract_geoip_asn(&root).await?;
        }
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
    println!("  rulekit all [--write]        Check source rules and extract GeoIP/ASN");
    println!("  rulekit normalize [--write]  Normalize source rule text files");
    println!("  rulekit geo        Extract GeoIP/ASN CIDR outputs");
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
            rules += normalized.lines().filter(|line| !line.trim().is_empty()).count();
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
