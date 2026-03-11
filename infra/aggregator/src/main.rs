use anyhow::Result;
use ipnet::{Ipv4Net, Ipv6Net};
use ipnetwork::IpNetwork;
use maxminddb::Reader;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
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
}

struct MmdbSource {
    name: &'static str,
    url: &'static str,
    r#type: &'static str,
}

const MMDB_SOURCES: &[MmdbSource] = &[
    MmdbSource { name: "ipinfo.country", url: "https://github.com/xream/geoip/releases/latest/download/ipinfo.country.mmdb", r#type: "country" },
    MmdbSource { name: "ip2location.country", url: "https://github.com/xream/geoip/releases/latest/download/ip2location.country.mmdb", r#type: "country" },
    MmdbSource { name: "ipinfo.asn", url: "https://github.com/xream/geoip/releases/latest/download/ipinfo.asn.mmdb", r#type: "asn" },
    MmdbSource { name: "ip2location.asn", url: "https://github.com/xream/geoip/releases/latest/download/ip2location.asn.mmdb", r#type: "asn" },
];

struct AsnTarget {
    name: &'static str,
    asns: &'static [u32],
}

const ASN_TARGETS: &[AsnTarget] = &[
    AsnTarget { name: "asn-cloudflare", asns: &[13335] },
    AsnTarget { name: "asn-google", asns: &[15169, 396982] },
    AsnTarget { name: "asn-microsoft", asns: &[8075] },
    AsnTarget { name: "asn-amazon", asns: &[16509, 14618] },
    AsnTarget { name: "asn-facebook", asns: &[32934] },
    AsnTarget { name: "asn-telegram", asns: &[62041, 62014, 59930, 44907] },
    AsnTarget { name: "asn-netflix", asns: &[2906] },
    AsnTarget { name: "asn-github", asns: &[36459] },
    AsnTarget { name: "asn-twitter", asns: &[13414] },
    AsnTarget { name: "asn-apple", asns: &[714, 6185] },
    AsnTarget { name: "asn-discord", asns: &[49544] },
    AsnTarget { name: "asn-spotify", asns: &[8403] },
    AsnTarget { name: "asn-steam", asns: &[32590] },
    AsnTarget { name: "asn-disney", asns: &[19679] },
    AsnTarget { name: "asn-oracle", asns: &[31898] },
    AsnTarget { name: "asn-akamai", asns: &[16625, 20940, 3131, 33905, 34164, 34850, 43639, 53235, 54104] },
    AsnTarget { name: "asn-twitch", asns: &[46489] },
    AsnTarget { name: "asn-alibaba", asns: &[37963, 45102, 132335] },
    AsnTarget { name: "asn-tencent", asns: &[132203, 132591, 133478, 133543] },
    AsnTarget { name: "asn-bytedance", asns: &[138690] },
    AsnTarget { name: "asn-baidu", asns: &[55967, 134177] },
];

#[tokio::main]
async fn main() -> Result<()> {
    let root = find_root();
    let ip_dir = root.join("source").join("ip");
    let tmp_dir = root.join(".mmdb-cache");

    fs::create_dir_all(&ip_dir)?;
    fs::create_dir_all(&tmp_dir)?;

    println!("============================================================");
    println!("  Gins-Rules MMDB Parser (Rust Extreme Version)");
    println!("============================================================");

    let mut country_cidrs: HashMap<String, HashSet<IpNetwork>> = HashMap::new();
    let mut asn_cidrs: HashMap<u32, HashSet<IpNetwork>> = HashMap::new();

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

    let common_regions: HashSet<&str> = vec!["CN", "SG", "TW", "JP"].into_iter().collect();
    let mut not_cn = HashSet::new();

    for (code, nets) in &country_cidrs {
        if code != "CN" {
            for net in nets { not_cn.insert(*net); }
        }
        if common_regions.contains(code.as_str()) {
            let out_path = ip_dir.join(format!("{}.txt", code.to_lowercase()));
            write_aggregated_ip_list(&out_path, nets)?;
            println!("  [WRITE] ip/{}.txt: {} networks", code.to_lowercase(), nets.len());
        }
    }

    if !not_cn.is_empty() {
        write_aggregated_ip_list(&ip_dir.join("!cn.txt"), &not_cn)?;
        println!("  [WRITE] ip/!cn.txt: {} networks (All non-CN)", not_cn.len());
    }

    for target in ASN_TARGETS {
        let mut merged = HashSet::new();
        for &asn in target.asns {
            if let Some(nets) = asn_cidrs.get(&asn) {
                for net in nets { merged.insert(*net); }
            }
        }
        if !merged.is_empty() {
            write_aggregated_ip_list(&ip_dir.join(format!("{}.txt", target.name)), &merged)?;
            println!("  [WRITE] ip/{}.txt: {} networks", target.name, merged.len());
        }
    }

    println!("============================================================");
    Ok(())
}

fn extract_country_cidrs(reader: &Reader<Vec<u8>>, storage: &mut HashMap<String, HashSet<IpNetwork>>) -> Result<usize> {
    let mut count = 0;
    let iter = reader.networks(Default::default())?;
    for result in iter {
        let lookup = result?;
        if let Some(code) = lookup.decode::<CountryRecord>()?.and_then(|r| r.country).and_then(|c| c.iso_code) {
            storage.entry(code).or_default().insert(lookup.network()?);
            count += 1;
        }
    }
    Ok(count)
}

fn extract_asn_cidrs(reader: &Reader<Vec<u8>>, storage: &mut HashMap<u32, HashSet<IpNetwork>>) -> Result<usize> {
    let mut count = 0;
    let iter = reader.networks(Default::default())?;
    for result in iter {
        let lookup = result?;
        if let Some(asn) = lookup.decode::<AsnRecord>()?.and_then(|r| r.autonomous_system_number) {
            storage.entry(asn).or_default().insert(lookup.network()?);
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
    let file = fs::File::create(path)?;
    let mut writer = BufWriter::new(file);
    for n in Ipv4Net::aggregate(&v4) { writeln!(writer, "{}", n)?; }
    for n in Ipv6Net::aggregate(&v6) { writeln!(writer, "{}", n)?; }
    writer.flush()?;
    Ok(())
}

fn find_root() -> PathBuf {
    let mut curr = std::env::current_dir().unwrap();
    while curr.parent().is_some() {
        if curr.join(".git").exists() { return curr; }
        curr = curr.parent().unwrap().to_path_buf();
    }
    curr
}
