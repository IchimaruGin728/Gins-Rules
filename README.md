# Gins-Rules

Comprehensive, self-maintained proxy rule list repository with fine-grained per-service categorization.

## Features
- **90+ Services**: Including Apple, Microsoft, Google, OpenAI, TikTok, etc.
- **GeoIP & ASN**: **Full Globe Coverage** (auto-extracted from MMDB: ipinfo + ip2location via [xream/geoip](https://github.com/xream/geoip))
- **Fallback Logic**: Specialized `!cn.txt` for all non-China IPs, providing a global fallback for rare regions.
- **Multi-Format Support**: 
    - **sing-box**: `.srs` (binary), `.json`
    - **Mihomo (Clash Meta) / Stash**: `.mrs` (binary), `.yaml`
    - **Surge / Loon / Shadowrocket**: `.list` (Surge format)
    - **QuantumultX**: `.list` (Native format)
    - **Egern**: `.yaml` (Native format)
- **High Performance**: Pre-compiled binary rules for modern clients.
- **Automated**: Multi-source upstream sync + MMDB extraction + auto compile via GitHub Actions.
- **Global Distribution**: Hosted on Cloudflare Pages for maximum speed and stability.

## Subscription

### 1. Merged (All services in one link)

| Category | Format | URL |
|----------|--------|-----|
| Proxy All | List | `https://rules-api.ichimarugin728.dev/ruleset/proxy.list` |
| Direct All | List | `https://rules-api.ichimarugin728.dev/ruleset/direct.list` |
| Reject All | List | `https://rules-api.ichimarugin728.dev/ruleset/reject.list` |
| IP All | List | `https://rules-api.ichimarugin728.dev/ruleset/ip.list` |

### 2. Individual Service

| Client | Format | URL Pattern |
|--------|--------|-------------|
| sing-box | SRS | `https://rules.ichimarugin728.dev/singbox/proxy/apple.srs` |
| Mihomo | MRS | `https://rules.ichimarugin728.dev/mihomo/proxy/apple.mrs` |
| Surge / Loon | List | `https://rules.ichimarugin728.dev/text/proxy/apple.list` |
| QuantumultX | List | `https://rules.ichimarugin728.dev/quanx/proxy/apple.list` |
| Egern | YAML | `https://rules.ichimarugin728.dev/egern/proxy/apple.yaml` |

## Complete Service List

### Proxy (61 services)

| Service | File | Description |
|---------|------|-------------|
| AI Other | `ai-other.txt` | Other AI services |
| Amazon | `amazon.txt` | Amazon services |
| Apple | `apple.txt` | Apple services |
| Apple Music | `apple-music.txt` | Apple Music streaming |
| Apple TV | `appletv.txt` | Apple TV+ |
| Blizzard | `blizzard.txt` | Blizzard Entertainment |
| Claude | `claude.txt` | Anthropic Claude AI |
| Cloudflare | `cloudflare.txt` | Cloudflare services |
| Copilot | `copilot.txt` | GitHub/Microsoft Copilot |
| Discord | `discord.txt` | Discord |
| Disney+ | `disney.txt` | Disney Plus |
| Docker | `docker.txt` | Docker Hub & Registry |
| Dropbox | `dropbox.txt` | Dropbox |
| DuckDuckGo | `duckduckgo.txt` | DuckDuckGo search |
| Epic | `epic.txt` | Epic Games |
| Facebook | `facebook.txt` | Facebook / Meta |
| Gemini | `gemini.txt` | Google Gemini AI |
| GFW List | `gfw-list.txt` | GFW blocked domains (merged) |
| GitHub | `github.txt` | GitHub |
| GitLab | `gitlab.txt` | GitLab |
| Google | `google.txt` | Google services |
| HBO | `hbo.txt` | HBO Max |
| Instagram | `instagram.txt` | Instagram |
| LINE | `line.txt` | LINE messaging |
| LinkedIn | `linkedin.txt` | LinkedIn |
| Medium | `medium.txt` | Medium publishing |
| Microsoft | `microsoft.txt` | Microsoft services |
| miHoYo Intl | `mihoyo-intl.txt` | miHoYo international |
| Netflix | `netflix.txt` | Netflix |
| News Intl | `news-intl.txt` | International news |
| Nintendo | `nintendo.txt` | Nintendo |
| Notion | `notion.txt` | Notion |
| NPM | `npm.txt` | NPM registry |
| OneDrive | `onedrive.txt` | Microsoft OneDrive |
| OpenAI | `openai.txt` | OpenAI / ChatGPT |
| PayPal | `paypal.txt` | PayPal |
| Pinterest | `pinterest.txt` | Pinterest |
| PlayStation | `playstation.txt` | PlayStation Network |
| Prime Video | `primevideo.txt` | Amazon Prime Video |
| ProtonMail | `protonmail.txt` | ProtonMail |
| Proxy List | `proxy-list.txt` | General proxy domains (merged) |
| Reddit | `reddit.txt` | Reddit |
| Riot Games | `riot.txt` | Riot Games |
| Signal | `signal.txt` | Signal messenger |
| Slack | `slack.txt` | Slack |
| Snapchat | `snapchat.txt` | Snapchat |
| Speedtest | `speedtest.txt` | Speedtest.net |
| Spotify | `spotify.txt` | Spotify |
| Stack Overflow | `stackoverflow.txt` | Stack Overflow |
| Steam | `steam.txt` | Steam |
| Telegram | `telegram.txt` | Telegram |
| Threads | `threads.txt` | Meta Threads |
| TikTok | `tiktok.txt` | TikTok |
| Twitch | `twitch.txt` | Twitch |
| Twitter/X | `twitter.txt` | Twitter / X |
| Vercel | `vercel.txt` | Vercel |
| WhatsApp | `whatsapp.txt` | WhatsApp |
| Wikipedia | `wikipedia.txt` | Wikipedia |
| Xbox | `xbox.txt` | Xbox Live |
| YouTube | `youtube.txt` | YouTube |
| Zoom | `zoom.txt` | Zoom |

### Direct (24 services)

| Service | File | Description |
|---------|------|-------------|
| Alibaba | `alibaba.txt` | Alibaba Group |
| Apple CDN | `apple-cdn.txt` | Apple CDN endpoints |
| Baidu | `baidu.txt` | Baidu |
| Bilibili | `bilibili.txt` | Bilibili |
| Bypass Domain | `bypass-domain.txt` | Bypass domains |
| Bypass IP | `bypass-ip.txt` | Bypass IPs |
| ByteDance | `bytedance.txt` | ByteDance / Douyin |
| CN CDN | `cn-cdn.txt` | China CDN providers |
| CN Other | `cn-other.txt` | Other China domains |
| DiDi | `didi.txt` | DiDi |
| Direct List | `direct-list.txt` | General direct domains (merged) |
| Douyu/Huya | `douyu-huya.txt` | Douyu & Huya live streaming |
| JD | `jd.txt` | JD.com |
| Kuaishou | `kuaishou.txt` | Kuaishou |
| Meituan | `meituan.txt` | Meituan |
| Microsoft CDN | `microsoft-cdn.txt` | Microsoft CDN endpoints |
| miHoYo | `mihoyo.txt` | miHoYo China |
| NetEase | `netease.txt` | NetEase |
| Pinduoduo | `pinduoduo.txt` | Pinduoduo |
| Steam CDN | `steam-cdn.txt` | Steam CDN (direct) |
| Tencent | `tencent.txt` | Tencent services |
| Weibo | `weibo.txt` | Weibo |
| Xiaomi | `xiaomi.txt` | Xiaomi |
| Zhihu | `zhihu.txt` | Zhihu |

### Reject (4 lists)

| Service | File | Description |
|---------|------|-------------|
| Ads | `ads.txt` | Ad domains |
| Privacy | `privacy.txt` | Privacy-invasive trackers |
| Reject List | `reject-list.txt` | General reject domains (merged) |
| Tracking | `tracking.txt` | Tracking domains |

### IP / GeoIP / ASN (15 lists)

> Auto-extracted from MMDB via `cmd/mmdb` (multi-source: ipinfo + ip2location merged & deduplicated)

| List | File | Source | Description |
|------|------|--------|-------------|
| China IP | `cn.txt` | GeoIP MMDB | 78,000+ China IP CIDRs |
| Non-China IP | `!cn.txt` | GeoIP MMDB | **Full Globe Fallback** (All CIDRs NOT in CN) |
| Private IP | `private.txt` | GeoIP MMDB | RFC1918 private ranges |
| Telegram IP | `telegram.txt` | Manual | Telegram IP CIDRs |
| ASN Cloudflare | `asn-cloudflare.txt` | ASN MMDB | AS13335 |
| ASN Google | `asn-google.txt` | ASN MMDB | AS15169, AS396982 |
| ASN Microsoft | `asn-microsoft.txt` | ASN MMDB | AS8075 |
| ASN Amazon | `asn-amazon.txt` | ASN MMDB | AS16509, AS14618 |
| ASN Meta | `asn-facebook.txt` | ASN MMDB | AS32934 |
| ASN Telegram | `asn-telegram.txt` | ASN MMDB | AS62041, AS62014, AS59930, AS44907 |
| ASN Netflix | `asn-netflix.txt` | ASN MMDB | AS2906 |
| ASN GitHub | `asn-github.txt` | ASN MMDB | AS36459 |
| ASN Twitter/X | `asn-twitter.txt` | ASN MMDB | AS13414 |
| ASN Apple | `asn-apple.txt` | ASN MMDB | AS714, AS6185 |
| ASN Discord | `asn-discord.txt` | ASN MMDB | AS49544 |
| ASN Spotify | `asn-spotify.txt` | ASN MMDB | AS8403 |
| ASN Steam/Valve | `asn-steam.txt` | ASN MMDB | AS32590 |
| ASN Disney+ | `asn-disney.txt` | ASN MMDB | AS19679 |
| ASN Oracle | `asn-oracle.txt` | ASN MMDB | AS31898 |
| ASN Akamai | `asn-akamai.txt` | ASN MMDB | AS16625, AS20940, AS3131... |
| ASN Alibaba | `asn-alibaba.txt` | ASN MMDB | AS37963, AS45102, AS132335 |
| ASN Tencent | `asn-tencent.txt` | ASN MMDB | AS132203, AS132591, AS133478, AS133543 |
| ASN ByteDance | `asn-bytedance.txt` | ASN MMDB | AS138690 |
| ASN Baidu | `asn-baidu.txt` | ASN MMDB | AS55967, AS134177 |

## Upstream Sources

| Name | Source | Category | Notes |
|------|--------|----------|-------|
| Loyalsoldier proxy-list | [v2ray-rules-dat](https://github.com/Loyalsoldier/v2ray-rules-dat) | proxy | Domain list |
| MetaCubeX proxy | [meta-rules-dat](https://github.com/MetaCubeX/meta-rules-dat) | proxy | Domain list |
| Loyalsoldier direct-list | [v2ray-rules-dat](https://github.com/Loyalsoldier/v2ray-rules-dat) | direct | Domain list |
| MetaCubeX cn | [meta-rules-dat](https://github.com/MetaCubeX/meta-rules-dat) | direct | China domains |
| Loyalsoldier reject-list | [v2ray-rules-dat](https://github.com/Loyalsoldier/v2ray-rules-dat) | reject | Ad/tracker list |
| MetaCubeX category-ads-all | [meta-rules-dat](https://github.com/MetaCubeX/meta-rules-dat) | reject | Ad list |
| Loyalsoldier GFW | [v2ray-rules-dat](https://github.com/Loyalsoldier/v2ray-rules-dat) | proxy | GFW list |
| MetaCubeX GFW | [meta-rules-dat](https://github.com/MetaCubeX/meta-rules-dat) | proxy | GFW list |
| Blackmatrix7 | [ios_rule_script](https://github.com/blackmatrix7/ios_rule_script) | proxy | YouTube, Netflix, Disney+, Discord, Telegram, TikTok |
| **ipinfo MMDB** | [xream/geoip](https://github.com/xream/geoip) | ip | Country + ASN (MMDB) |
| **ip2location MMDB** | [xream/geoip](https://github.com/xream/geoip) | ip | Country + ASN (MMDB) |

> All upstream sources are **multi-source merged & deduplicated** — same-target rules from different providers are combined and deduped automatically.

## Repository Structure
- `source/`: Plain text core rule lists (no comments).
  - `source/proxy/`: 61 proxy service rule files
  - `source/direct/`: 24 direct service rule files
  - `source/reject/`: 4 reject rule files
  - `source/ip/`: 15 IP/GeoIP/ASN rule files (auto-generated from MMDB)
  - `source/upstream/`: Auto-synced upstream rule files
  - `source/sources.json`: Upstream source configuration
- `cmd/sync/`: Go upstream rule fetcher (multi-source merge).
- `cmd/mmdb/`: Go MMDB parser (downloads ipinfo + ip2location MMDB → extracts GeoIP/ASN → txt).
- `cmd/compile/`: Go compiler for multi-format output (8 formats + manifests).
- `cmd/scanner/`: CF Worker for automated scanning, AI classification, and rule merging.
- `compiled/`: Auto-generated rules for distribution (hierarchical structure).

## Development
```bash
# Sync upstream rules
go run ./cmd/sync/

# Extract GeoIP & ASN from MMDB
go run ./cmd/mmdb/

# Compile rules locally
go run ./cmd/compile/
```

## Credits
- [xream/geoip](https://github.com/xream/geoip) — GeoIP & ASN MMDB (ipinfo + ip2location)
- [Loyalsoldier/v2ray-rules-dat](https://github.com/Loyalsoldier/v2ray-rules-dat) — Domain rule lists
- [MetaCubeX/meta-rules-dat](https://github.com/MetaCubeX/meta-rules-dat) — Domain rule lists
- [Blackmatrix7/ios_rule_script](https://github.com/blackmatrix7/ios_rule_script) — Service-specific rules
- Official Enterprise Documentation (Apple, Microsoft, Docker, etc.)
