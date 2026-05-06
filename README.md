# Gins-Rules

Comprehensive, self-maintained proxy rule list repository with high-performance R2-backed distribution. Focused on the absolute bleeding edge of routing protocol support.

## ✨ Features

- **Dynamic ASN Routing**: Native support for **Autonomous System Numbers (ASN)** across all platforms (Sing-box, Mihomo, Loon, QX, etc.).
- **AI Rule-sets**: Dedicated high-priority routing for **Claude**, **Gemini**, and **Copilot** (merged into `ai-other`).
- **Rolling Xray Kernel**: Automatically tracks and compiles with the latest **xtls/xray-core** (Pinned by Commit SHA) for dev-release feature parity.
- **Dynamic R2 Storage**: Powered by **Cloudflare R2** via S3 protocol. Handles full-scale IP/ASN rulesets (30MB+) with differential sync.
- **High-Speed Sync**: GitHub Actions optimized with **AWS CLI S3 Sync** for lightning-fast delivery.
- **Premium Icon Hub**: A centralized, automated aggregator of **30+ premium icon libraries** (Qure, Lige, Semporia, etc.) with a searchable dashboard and click-to-copy functionality.
- **Multi-Format Matrix (12 Formats)**:
    - **sing-box**: `.srs` (binary v2), `.json`
    - **Mihomo (Clash Meta) / Stash**: `.mrs` (binary), `.yaml`
    - **Surfboard**: `.list` (Standard Ruleset), `.txt` (Optimized Domainset)
    - **Exclave**: `.list` (Standard Route format)
    - **Surge / Shadowrocket**: `.list` (Standard format)
    - **Loon**: `.lsr` (Loon-specific standard rule format)
    - **QuantumultX**: `.list` (Native policy format)
    - **Egern**: `.yaml` (Multi-category config)
- **Bleeding Edge Binaries**: automated build using the latest **Sing-box (Pre-release)** and **Mihomo (Alpha)** compilers.

## 🚀 Subscription Matrix

The delivery URLs are backed by high-performance R2 storage with Cloudflare Smart Routing.

| Client | Format | Base Path Pattern | Example URL (Proxy/Apple) |
|--------|--------|-------------------|--------------------------|
| **sing-box** | SRS | `/ruleset/singbox/{cat}/{name}.srs` | [apple.srs](https://rules.ichimarugin728.dev/ruleset/singbox/proxy/apple.srs) |
| **Mihomo** | MRS | `/ruleset/mihomo/{cat}/{name}.mrs` | [apple.mrs](https://rules.ichimarugin728.dev/ruleset/mihomo/proxy/apple.mrs) |
| **Stash** | MRS | `/ruleset/stash/{cat}/{name}.mrs` | [apple.mrs](https://rules.ichimarugin728.dev/ruleset/stash/proxy/apple.mrs) |
| **Surge** | List | `/ruleset/surge/{cat}/{name}.list` | [apple.list](https://rules.ichimarugin728.dev/ruleset/surge/proxy/apple.list) |
| **Loon** | LSR | `/ruleset/loon/{cat}/{name}.lsr` | [apple.lsr](https://rules.ichimarugin728.dev/ruleset/loon/proxy/apple.lsr) |
| **QuantumultX** | List | `/ruleset/quantumultx/{cat}/{name}.list` | [apple.list](https://rules.ichimarugin728.dev/ruleset/quantumultx/proxy/apple.list) |
| **Shadowrocket**| List | `/ruleset/shadowrocket/{cat}/{name}.list` | [apple.list](https://rules.ichimarugin728.dev/ruleset/shadowrocket/proxy/apple.list) |
| **Surfboard** | List | `/ruleset/surfboard/{cat}/{name}.list` | [apple.list](https://rules.ichimarugin728.dev/ruleset/surfboard/proxy/apple.list) |
| **Surfboard (Opt)**| TXT | `/ruleset/surfboard/{cat}/{name}.txt` | [apple.txt](https://rules.ichimarugin728.dev/ruleset/surfboard/proxy/apple.txt) |
| **Egern** | YAML | `/ruleset/egern/{cat}/{name}.yaml` | [apple.yaml](https://rules.ichimarugin728.dev/ruleset/egern/proxy/apple.yaml) |
| **Exclave** | Route | `/ruleset/exclave/{cat}/{name}.list` | [apple.list](https://rules.ichimarugin728.dev/ruleset/exclave/proxy/apple.list) |
| **Icons Hub** | JSON | `/Gins-Icons.json` | [Gins-Icons.json](https://rules.ichimarugin728.dev/Gins-Icons.json) |

### 🎨 Icon Hub Dashboard
Access the premium, searchable icon collection at: **[rules.ichimarugin728.dev/icons](https://rules.ichimarugin728.dev/icons)**

### 📂 Available Categories (`{cat}`)

- `proxy`: Global services (Google, Apple, Telegram, etc.)
- `direct`: Bypass/Domestic routing (CN sites, Private IPs, etc.)
- `reject`: Ads, Tracking, and Telemetry.
- `ip`: Country-specific CIDR lists (e.g. `cn`, `us`, `!cn`).
- `asn`: Network-specific ASN lists (e.g. `asn-google`, `asn-cloudflare`).
- `ai-other`: High-priority AI services (Claude, Gemini, Copilot).

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
- **cmd/sync/**: Go upstream rule fetcher (Supports ASN & Domain mapping).
- **cmd/compile/**: Go compiler for multi-format output. Features native Xray .dat bundling.
- **cmd/icons/**: High-performance icon aggregator (30+ sources, SHA-256 fingerprinting).
- **cmd/scanner/**: High-performance CF Worker gateway and dashboard.
- `bin/`: Local workspace for Alpha/Beta binary compilers.

## ⚙️ Development

```bash
# 1. Sync upstream rules (Recognizes ASN and AI sources)
go run ./cmd/sync/

# 2. Sync premium icons (Aggregates 30+ libraries)
go run ./cmd/icons/

# 3. Compile rules locally (Requires Go 1.24+)
go run ./cmd/compile

# 4. Dashboard Development (pnpm)
cd dashboard
pnpm install
pnpm dev
```

## 📅 Maintenance Schedule

- **Automated Sync**: Twice daily at **07:28 SGT** and **17:16 SGT**.
- **Smart Update**: Build process only triggers when source data or core kernels change.

## 🤝 Credits

- [xream/geoip](https://github.com/xream/geoip) — Primary GeoIP & ASN source.
- [blackmatrix7/ios_rule_script](https://github.com/blackmatrix7/ios_rule_script) — High-quality service rule sources.
- [fmz200/wool_scripts](https://github.com/fmz200/wool_scripts) — Comprehensive Loon/Surge rule collections.
- [Loyalsoldier/v2ray-rules-dat](https://github.com/Loyalsoldier/v2ray-rules-dat).
- [Rabbit-Spec/Surge](https://github.com/Rabbit-Spec/Surge) — Apple, China, China ASN/CIDR, and China media source rules.
- [Centralmatrix3/Matrix-io](https://github.com/Centralmatrix3/Matrix-io) — Multi-service Surge rule source used for deduped supplemental rules.
- Powered by **Cloudflare Workers** & **R2 Storage**.
