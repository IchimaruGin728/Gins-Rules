use compact_str::CompactString;
use std::collections::{HashMap, HashSet};

use crate::models::RuleSet;

/// Optimize a RuleSet in-place:
/// 1. Expand DOMAIN-KEYWORD → known DOMAIN-SUFFIX (fast trie matching)
/// 2. Expand DOMAIN-WILDCARD → known DOMAIN-SUFFIX
/// 3. Extract DOMAIN-REGEX → DOMAIN-SUFFIX when possible
/// 4. Sort all rule vectors by performance tier
pub fn optimize(rules: &mut RuleSet) {
    let keyword_db = build_keyword_db();

    expand_keywords(rules, &keyword_db);
    expand_wildcards(rules, &keyword_db);
    expand_regex(rules);
    sort_by_performance(rules);
}

/// Sort every vector in the RuleSet alphabetically.
///
/// Domain rules: HOST (exact) → HOST-SUFFIX → HOST-KEYWORD/WILDCARD → REGEX
/// IP rules:     IP-CIDR → IP-ASN
/// Mixed:        domain rules first, then IP rules, then process/user-agent
pub fn sort_by_performance(rules: &mut RuleSet) {
    // Domain: exact before suffix
    let mut ds: Vec<_> = rules.domain.iter().cloned().collect();
    ds.sort();
    rules.domain = ds.into_iter().collect();

    let mut dsa: Vec<_> = rules.domain_suffix.iter().cloned().collect();
    dsa.sort();
    rules.domain_suffix = dsa.into_iter().collect();

    let mut dk: Vec<_> = rules.domain_keyword.iter().cloned().collect();
    dk.sort();
    rules.domain_keyword = dk.into_iter().collect();

    let mut dw: Vec<_> = rules.domain_wildcard.iter().cloned().collect();
    dw.sort();
    rules.domain_wildcard = dw.into_iter().collect();

    let mut dr: Vec<_> = rules.domain_regex.iter().cloned().collect();
    dr.sort();
    rules.domain_regex = dr.into_iter().collect();

    // IP
    let mut ip: Vec<_> = rules.ip_cidr.iter().cloned().collect();
    ip.sort();
    rules.ip_cidr = ip.into_iter().collect();

    let mut asn: Vec<_> = rules.ip_asn.iter().cloned().collect();
    asn.sort();
    rules.ip_asn = asn.into_iter().collect();

    // Process / UA
    let mut pn: Vec<_> = rules.process_name.iter().cloned().collect();
    pn.sort();
    rules.process_name = pn.into_iter().collect();

    let mut ua: Vec<_> = rules.user_agent.iter().cloned().collect();
    ua.sort();
    rules.user_agent = ua.into_iter().collect();
}

// ── Keyword expansion ──────────────────────────────────────────────

fn expand_keywords(rules: &mut RuleSet, db: &HashMap<&str, Vec<&str>>) {
    if rules.domain_keyword.is_empty() {
        return;
    }

    let existing: HashSet<_> = rules.domain_suffix.iter().map(|s| s.as_str()).collect();
    let mut new_suffixes: Vec<CompactString> = Vec::new();

    for kw in &rules.domain_keyword {
        if let Some(expansions) = db.get(kw.as_str()) {
            for suffix in expansions {
                if !existing.contains(suffix) {
                    new_suffixes.push(CompactString::new(suffix));
                }
            }
        }
    }

    rules.domain_suffix.extend(new_suffixes);
}

// ── Wildcard expansion ─────────────────────────────────────────────

fn expand_wildcards(rules: &mut RuleSet, db: &HashMap<&str, Vec<&str>>) {
    if rules.domain_wildcard.is_empty() {
        return;
    }

    let existing: HashSet<_> = rules.domain_suffix.iter().map(|s| s.as_str()).collect();
    let mut new_suffixes: Vec<CompactString> = Vec::new();

    for wc in &rules.domain_wildcard {
        if let Some(suffix) = wildcard_to_suffix(wc) {
            if let Some(expansions) = db.get(suffix) {
                for exp in expansions {
                    if !existing.contains(exp) {
                        new_suffixes.push(CompactString::new(exp));
                    }
                }
            }
        }
    }

    rules.domain_suffix.extend(new_suffixes);
}

fn wildcard_to_suffix(wc: &str) -> Option<&str> {
    if let Some(rest) = wc.strip_prefix("*.") {
        if !rest.contains('*') {
            return Some(rest);
        }
    }
    None
}

// ── Regex → suffix extraction ──────────────────────────────────────

fn expand_regex(rules: &mut RuleSet) {
    if rules.domain_regex.is_empty() {
        return;
    }

    let existing: HashSet<_> = rules.domain_suffix.iter().map(|s| s.as_str()).collect();
    let mut new_suffixes: Vec<CompactString> = Vec::new();

    for rx in &rules.domain_regex {
        if let Some(suffix) = simple_regex_to_suffix(rx) {
            if !existing.contains(suffix) {
                new_suffixes.push(CompactString::new(suffix));
            }
        }
    }

    rules.domain_suffix.extend(new_suffixes);
}

/// Detect simple regex patterns like `^.*\.example\.com$` and extract the suffix.
fn simple_regex_to_suffix(rx: &str) -> Option<&str> {
    // Pattern: ^.*\.example\.com$  or  ^(.+\.)?example\.com$
    let rx = rx.trim();

    if let Some(inner) = rx.strip_prefix("^.*\\.").and_then(|s| s.strip_suffix('$')) {
        let domain = inner.replace("\\.", ".");
        if !domain.contains('\\') && domain.contains('.') {
            return Some(Box::leak(domain.into_boxed_str()));
        }
    }

    None
}

// ── Keyword → known suffix database ────────────────────────────────
//
// These are well-known domain families. When a keyword rule like
// `keyword:google` is encountered, we expand it into DOMAIN-SUFFIX
// rules for all known Google TLDs so the trie can match them in O(n).
// The original keyword rule is kept as a catch-all fallback.

fn build_keyword_db() -> HashMap<&'static str, Vec<&'static str>> {
    let mut m: HashMap<&str, Vec<&str>> = HashMap::new();

    m.insert("google", vec![
        "google.com", "google.com.hk", "google.com.sg", "google.com.tw",
        "google.co.jp", "google.co.kr", "google.dev",
        "googleapis.com", "googleapis.cn",
        "googlecloud.com", "googlecode.com", "googledomains.com",
        "googlehosted.com", "googlemail.com", "googleoptimize.com",
        "googlesource.com", "googlesyndication.com", "googletagmanager.com",
        "googleusercontent.com", "googlevideo.com", "google-analytics.com",
        "googleadservices.com", "withgoogle.com",
        "blog.google", "deepmind.google", "firebase.google.com",
    ]);

    m.insert("gmail", vec![
        "googlemail.com", "google.com", "google.com.hk",
        "google.com.sg", "google.com.tw",
    ]);

    m.insert("youtube", vec![
        "youtube.com", "youtu.be", "ytimg.com", "googlevideo.com",
        "youtube-nocookie.com", "youtube-ui.l.google.com",
    ]);

    m.insert("facebook", vec![
        "facebook.com", "facebook.net", "fbcdn.net", "fbsbx.com",
        "fb.com", "facebookwlmail.com",
    ]);

    m.insert("fbcdn", vec!["fbcdn.net", "fbsbx.com"]);

    m.insert("twitter", vec![
        "twitter.com", "twimg.com", "t.co", "x.com",
    ]);

    m.insert("instagram", vec![
        "instagram.com", "cdninstagram.com", "ig.me",
    ]);

    m.insert("whatsapp", vec![
        "whatsapp.com", "whatsapp.net", "whatsapp-cdn.com",
    ]);

    m.insert("tiktok", vec![
        "tiktok.com", "tiktokcdn.com", "tiktokv.com",
        "musical.ly", "muscdn.com", "byteoversea.com",
        "ibytedtos.com", "ibyteimg.com", "bytedapm.com",
        "isnssdk.com", "bytedance.com",
    ]);

    m.insert("tiktokcdn", vec![
        "tiktokcdn.com", "byteoversea.com", "ibytedtos.com", "ibyteimg.com",
    ]);

    m.insert("musical.ly", vec!["musical.ly", "muscdn.com"]);

    m.insert("telegram", vec![
        "telegram.org", "telegram.me", "t.me",
        "telegram-cdn.org", "telegra.ph",
    ]);

    m.insert("spotify", vec![
        "spotify.com", "spotifycdn.com", "scdn.co",
        "spoti.fi", "spotify.design",
    ]);

    m.insert("-spotify-", vec![
        "spotify.com", "spotifycdn.com", "scdn.co",
    ]);

    m.insert("microsoft", vec![
        "microsoft.com", "microsoftonline.com", "microsofttranslator.com",
        "windows.com", "windowsupdate.com", "office.com", "office365.com",
        "office.net", "outlook.com", "live.com", "msn.com",
        "azure.com", "azureedge.net", "msedge.net",
        "skype.com", "bing.com", "msftconnecttest.com",
    ]);

    m.insert("1drv", vec![
        "1drv.com", "onedrive.com", "live.com",
        "sharepoint.com", "office.com",
    ]);

    m.insert("onedrive", vec!["onedrive.com", "1drv.com", "live.com"]);
    m.insert("skydrive", vec!["onedrive.com", "1drv.com", "live.com"]);

    m.insert("github", vec![
        "github.com", "github.io", "githubusercontent.com",
        "githubassets.com", "githubapp.com",
    ]);

    m.insert("gitlab", vec!["gitlab.com", "gitlab.io", "gitlab.net"]);

    m.insert("dropbox", vec![
        "dropbox.com", "dropboxstatic.com", "dropboxusercontent.com",
        "dropboxapi.com", "getdropbox.com",
    ]);

    m.insert("amazon", vec![
        "amazon.com", "amazon.co.jp", "amazon.co.uk", "amazon.de",
        "amazon.ca", "amazon.com.au", "amazon.fr", "amazon.it",
        "amazon.es", "amazon.in", "amazon.sg", "amazon.ae",
        "amazonaws.com", "cloudfront.net", "awsglobalaccelerator.com",
    ]);

    m.insert("avoddashs", vec![
        "amazon.com", "amazonvideo.com", "primevideo.com",
    ]);

    m.insert("netflix", vec![
        "netflix.com", "nflxvideo.net", "nflximg.net",
        "nflxext.com", "nflxso.net", "netflix.net",
    ]);

    m.insert("netflixdnstest", vec![
        "netflix.com", "nflxvideo.net", "nflximg.net",
    ]);

    m.insert("apiproxy-device-prod-nlb-", vec![
        "netflix.com", "nflxvideo.net",
    ]);

    m.insert("dualstack.apiproxy-", vec![
        "netflix.com", "nflxvideo.net",
    ]);

    m.insert("alipay", vec![
        "alipay.com", "alipayobjects.com", "alibaba.com",
    ]);

    m.insert("taobao", vec![
        "taobao.com", "tmall.com", "alicdn.com", "alibaba.com",
    ]);

    m.insert("weibo", vec![
        "weibo.com", "weibo.cn", "weibocdn.com",
    ]);

    m.insert("porn", vec!["pornhub.com", "xvideos.com", "xnxx.com"]);

    m.insert("paypal", vec!["paypal.com", "paypalobjects.com"]);

    m.insert("sci-hub", vec!["sci-hub.se", "sci-hub.st", "sci-hub.ru"]);

    m.insert("testflight", vec!["testflight.apple.com", "apple.com"]);

    m.insert("blogspot", vec!["blogspot.com", "blogger.com"]);

    m.insert("speedtest", vec![
        "speedtest.net", "ooklaserver.net", "speedtest.custom",
    ]);

    m.insert("officecdn", vec![
        "office.com", "office.net", "officecdn.microsoft.com",
    ]);

    m.insert("abema", vec!["abema.io", "abema.tv", "hayabusa.io"]);

    m.insert("smp-device", vec!["apple.com"]);

    m.insert("ttvnw", vec!["twitch.tv", "ttvnw.net", "jtvnw.net"]);

    m.insert("uk-live", vec!["bbc.co.uk", "bbc.com", "bbci.co.uk"]);

    m.insert("openai", vec![
        "openai.com", "chatgpt.com", "oaistatic.com", "oaiusercontent.com",
    ]);

    m.insert("1e100", vec!["google.com"]);

    m
}
