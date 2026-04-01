# Gins-Rules

Comprehensive, self-maintained proxy rule list repository with high-performance R2-backed distribution. Focused on the absolute bleeding edge of routing protocol support.

## ✨ Features

- **Dynamic ASN Routing**: Native support for **Autonomous System Numbers (ASN)** across all platforms (Sing-box, Mihomo, Loon, QX, etc.).
- **AI Rule-sets**: Dedicated high-priority routing for **Claude**, **Gemini**, and **Copilot** (merged into `ai-other`).
- **Rolling Xray Kernel**: Automatically tracks and compiles with the latest **xtls/xray-core** (Pinned by Commit SHA) for dev-release feature parity.
- **Dynamic R2 Storage**: Powered by **Cloudflare R2** via S3 protocol. Handles full-scale IP/ASN rulesets (30MB+) with differential sync.
- **High-Speed Sync**: GitHub Actions optimized with **AWS CLI S3 Sync** for lightning-fast delivery.
- **Multi-Format Matrix (12 Formats)**:
    - **sing-box**: `.srs` (binary v2), `.json`
    - **Mihomo (Clash Meta) / Stash**: `.mrs` (binary), `.yaml`
    - **Surfboard**: `.list` (Standard Ruleset), `.txt` (Optimized Domainset)
    - **Exclave**: `.list` (Standard Route format)
    - **Surge / Loon / Shadowrocket**: `.list` (Standard format)
    - **QuantumultX**: `.list` (Native policy format)
    - **Egern**: `.yaml` (Multi-category config)
- **Bleeding Edge Binaries**: automated build using the latest **Sing-box (Pre-release)** and **Mihomo (Alpha)** compilers.

## 🚀 Subscription

Delivery URLs are backed by high-performance R2 storage with Cloudflare Smart Routing.

| Client | Format | URL Pattern |
|--------|--------|-------------|
| sing-box | SRS | `https://rules.ichimarugin728.dev/ruleset/singbox/proxy/apple.srs` |
| Mihomo | MRS | `https://rules.ichimarugin728.dev/ruleset/mihomo/proxy/apple.mrs` |
| ASN / IP | List | `https://rules.ichimarugin728.dev/ruleset/text/asn/asn-cloudflare.list` |
| AI Other | List | `https://rules.ichimarugin728.dev/ruleset/text/proxy/ai-other.list` |

## 📊 Analytics & Coverage

| Category | Description | Status |
|----------|-------------|--------|
| **Proxy** | 60+ global services (Google, Netflix, Discord, etc.) | ✅ Active |
| **AI Other** | Claude, Gemini, Copilot (Merged from BlackMatrix7) | 🆕 New |
| **Direct** | Domestic services, bypass lists, and CN routing | ✅ Active |
| **Reject** | Privacy tracking, ads, and telemetry | ✅ Active |
| **IP / GeoIP** | Country CIDR lists (e.g., `cn`, `us`, `!cn`) | ✅ Full Scale |
| **ASN / Network** | Enterprise BGP ranges (e.g., `asn-cloudflare`, `asn-google`) | ✅ First Class |

## 🛠️ Repository Structure

- `source/`: Plain text core rule lists.
  - `source/[category]/`: Domain lists.
  - `source/ip/`: Source for both **IP CIDR** and **ASN** rules.
- `cmd/sync/`: Go upstream rule fetcher (Supports ASN & Domain mapping).
- `cmd/compile/`: Go compiler for multi-format output. Features native Xray .dat bundling.
- `cmd/scanner/`: High-performance CF Worker gateway and dashboard.
- `bin/`: Local workspace for Alpha/Beta binary compilers.

## ⚙️ Development

```bash
# Sync upstream rules (Recognizes ASN and AI sources)
go run ./cmd/sync/

# Compile rules locally (Requires Go 1.22+)
go run ./cmd/compile
```

## 📅 Maintenance Schedule

- **Automated Sync**: Twice daily at **07:28 SGT** and **17:16 SGT**.
- **Smart Update**: Build process only triggers when source data or core kernels change.

## 🤝 Credits

- [xream/geoip](https://github.com/xream/geoip) — Primary GeoIP & ASN source.
- [Blackmatrix7/ios_rule_script](https://github.com/blackmatrix7/ios_rule_script) — High-quality AI rule sources.
- [Loyalsoldier/v2ray-rules-dat](https://github.com/Loyalsoldier/v2ray-rules-dat).
- Powered by **Cloudflare Workers** & **R2 Storage**.
