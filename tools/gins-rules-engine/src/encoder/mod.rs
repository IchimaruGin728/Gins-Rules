pub mod anywhere;
pub mod egern;
pub mod exclave;
pub mod loon;
pub mod mihomo;
pub mod quantumultx;
pub mod singbox;
pub mod surfboard;
pub mod surge;
pub mod text;

use anyhow::Result;
use rayon::prelude::*;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use crate::intermediate;
use crate::models::RuleSet;
use crate::optimizer;
use crate::parser;

const CATEGORIES: &[&str] = &["proxy", "direct", "reject", "ip", "asn"];

pub fn run(root: &str, output: &str) -> Result<()> {
    let root_path = PathBuf::from(root).canonicalize()?;
    let out_dir = root_path.join(output).join("ruleset");

    println!("🚀 [Engine] Building rule matrix...");

    let all_categories: BTreeMap<String, BTreeMap<String, RuleSet>> = CATEGORIES
        .par_iter()
        .map(|&cat| {
            let local_dir = root_path.join(format!("source/{}", cat));
            let upstream_dir = root_path.join(format!("source/upstream/{}", cat));

            let mut rule_names: HashSet<String> = HashSet::new();
            gather_names(&local_dir, &mut rule_names);
            gather_names(&upstream_dir, &mut rule_names);

            let mut rule_names_vec: Vec<String> = rule_names.into_iter().collect();
            rule_names_vec.sort_unstable();

            let mut category_rules: BTreeMap<String, RuleSet> = BTreeMap::new();
            let mut category_aggregate = RuleSet::new();

            for name in &rule_names_vec {
                let mut rules = RuleSet::new();

                let local_path = local_dir.join(format!("{}.txt", name));
                if let Ok(r) = parser::parse_file(&local_path) {
                    rules.merge(&r);
                }

                let upstream_path = upstream_dir.join(format!("{}.txt", name));
                if let Ok(r) = parser::parse_file(&upstream_path) {
                    rules.merge(&r);
                }

                if rules.is_empty() {
                    continue;
                }

                // Optimize: expand keywords→suffixes, sort by performance
                optimizer::optimize(&mut rules);

                category_aggregate.merge(&rules);
                category_rules.insert(name.clone(), rules);
            }

            // Encode all formats for each rule
            for (name, rules) in &category_rules {
                encode_all_formats(name, rules, &out_dir, cat);
            }

            // Encode category aggregate
            if !category_aggregate.is_empty() {
                optimizer::optimize(&mut category_aggregate);
                println!("    📦 Compiling aggregate bundle: {}", cat);
                encode_all_formats(cat, &category_aggregate, &out_dir, cat);
            }

            (cat.to_string(), category_rules)
        })
        .collect();

    // Compile AI aggregate
    let mut ai_aggregate = RuleSet::new();
    for (cat, rules_map) in &all_categories {
        if cat == "proxy" || cat == "ai" {
            for (name, rules) in rules_map {
                if is_ai_rule(name) {
                    ai_aggregate.merge(rules);
                }
            }
        }
    }
    if !ai_aggregate.is_empty() {
        optimizer::optimize(&mut ai_aggregate);
        println!("    🤖 Compiling AI aggregate bundle: ai");
        encode_all_formats("ai", &ai_aggregate, &out_dir, "ai");
    }

    // Write intermediate.json to compiled/ (parent of ruleset/)
    let compiled_dir = out_dir.parent().unwrap_or(&out_dir);
    intermediate::write(&all_categories, compiled_dir)?;

    // Build summary — source-level stats
    let total_services: usize = all_categories.values().map(|m| m.len()).sum();
    let total_rules: usize = all_categories.values().flat_map(|m| m.values()).map(|r| r.len()).sum();
    let total_ips: usize = all_categories
        .values()
        .flat_map(|m| m.values())
        .map(|r| r.ip_cidr.len() + r.ip_asn.len())
        .sum();

    // Count compiled output files
    let mut compiled_files: u64 = 0;
    let mut compiled_rules: u64 = 0;
    for format_dir in &["surge", "quantumultx", "mihomo", "loon", "surfboard",
                        "egern", "singbox", "exclave", "anywhere", "stash",
                        "shadowrocket", "text"] {
        let dir = out_dir.join(format_dir);
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Ok(files) = std::fs::read_dir(&path) {
                        for f in files.flatten() {
                            let fp = f.path();
                            if fp.is_file() && fp.extension().map_or(false, |e| e == "list" || e == "lsr" || e == "yaml" || e == "json" || e == "txt") {
                                compiled_files += 1;
                                if let Ok(content) = std::fs::read_to_string(&fp) {
                                    compiled_rules += content.lines()
                                        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
                                        .count() as u64;
                                }
                            }
                        }
                    }
                }
            }
        }
        break; // Only count one format (surge) to avoid over-counting
    }

    let summary = serde_json::json!({
        "services": total_services,
        "rules": total_rules,
        "ipRules": total_ips,
        "compiled_files": compiled_files,
        "compiled_rules": compiled_rules,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    std::fs::write(out_dir.join("build-summary.json"), serde_json::to_string(&summary)?)?;

    println!(
        "✨ [Engine] All formats generated. Services: {}, Rules: {}, IP Rules: {}",
        total_services, total_rules, total_ips
    );

    Ok(())
}

fn gather_names(dir: &Path, names: &mut HashSet<String>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("txt") {
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    names.insert(name.to_string());
                }
            }
        }
    }
}

fn encode_all_formats(name: &str, rules: &RuleSet, out_dir: &Path, cat: &str) {
    let formats: Vec<(&str, fn(&str, &RuleSet, &Path, &str) -> Result<()>)> = vec![
        ("singbox", singbox::encode),
        ("mihomo", mihomo::encode),
        ("stash", surge::encode_stash),
        ("surge", surge::encode),
        ("shadowrocket", surge::encode_shadowrocket),
        ("loon", loon::encode),
        ("quantumultx", quantumultx::encode),
        ("surfboard", surfboard::encode),
        ("egern", egern::encode),
        ("exclave", exclave::encode),
        ("anywhere", anywhere::encode),
        ("text", text::encode),
    ];

    for (dir_name, encode_fn) in formats {
        let target_dir = out_dir.join(dir_name).join(cat);
        if let Err(e) = encode_fn(name, rules, &target_dir, cat) {
            eprintln!("Error encoding {} to {}: {}", name, dir_name, e);
        }
    }
}

fn is_ai_rule(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    [
        "openai", "claude", "gemini", "copilot", "ai-other", "apple-intelligence",
        "mistral", "deepseek", "character", "perplexity", "groq", "anthropic",
    ]
    .iter()
    .any(|k| name_lower.contains(k))
}
