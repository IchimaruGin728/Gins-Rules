# Gins-Rules

Comprehensive, self-maintained proxy rule list repository with high-performance R2-backed distribution.

## ✨ Features

- **90+ Services**: Including Apple, Microsoft, Google, OpenAI, TikTok, etc.
- **Dynamic R2 Storage**: Powered by **Cloudflare R2** via S3 protocol, removing the 25MB Workers Assets limit. Handles full-scale IP/ASN rulesets (30MB+) with ease.
- **High-Speed Sync**: GitHub Actions optimized with **AWS CLI S3 Sync (100 Concurrent Requests)** for lightning-fast, differential rule delivery.
- **Smart Routing**: Deployed on **Cloudflare Workers** with **Smart Routing** and latest compatibility date (2026-04-01) for global low-latency access.
- **GeoIP & ASN**: Full Globe Coverage (auto-extracted from MMDB: ipinfo + ip2location) with separate, refined categorization.
- **Multi-Format Support**:
    - **sing-box**: `.srs` (binary), `.json`
    - **Mihomo (Clash Meta) / Stash**: `.mrs` (binary), `.yaml`
    - **Surge / Loon / Shadowrocket**: `.list` (Standard format)
    - **QuantumultX**: `.list` (Native format)
    - **Egern**: `.yaml` (Native format)
- **Bleeding Edge**: Automated build using the absolute latest **Sing-box (Alpha)** and **Mihomo (Alpha)** compilers for maximum feature support.

## 🚀 Subscription

The delivery URLs remain consistent, now backed by high-performance R2 storage.

| Client | Format | URL Pattern |
|--------|--------|-------------|
| sing-box | SRS | `https://rules.ichimarugin728.dev/ruleset/singbox/proxy/apple.srs` |
| Mihomo | MRS | `https://rules.ichimarugin728.dev/ruleset/mihomo/proxy/apple.mrs` |
| Surge / Loon | List | `https://rules.ichimarugin728.dev/ruleset/text/proxy/apple.list` |
| QuantumultX | List | `https://rules.ichimarugin728.dev/ruleset/quanx/proxy/apple.list` |

## 📊 Analytics & Coverage

| Category | Description | Status |
|----------|-------------|--------|
| **Proxy** | 60+ global services merged & deduplicated | ✅ Active |
| **Direct** | 20+ domestic services and bypass lists | ✅ Active |
| **Reject** | Privacy tracking, ads, and telemetry | ✅ Active |
| **IP / GeoIP** | Full country CIDR lists (e.g., `cn`, `us`, `!cn`) | ✅ Full Scale (R2) |
| **ASN / Network** | Enterprise network ranges (e.g., `asn-cloudflare`, `asn-apple`) | ✅ Refined |

## 🛠️ Repository Structure

- `source/`: Plain text core rule lists (no comments).
  - `source/proxy/`, `source/direct/`, `source/reject/`: Domain lists.
  - `source/ip/`: Combined directory for both **IP CIDR** and **ASN** source files.
- `cmd/sync/`: Go upstream rule fetcher (multi-source merge).
- `cmd/compile/`: Go compiler for multi-format output. Smartly categorizes files into `ip` or `asn` directories based on prefixes.
- `cmd/scanner/`: High-performance CF Worker serving the dashboard and R2 gateway.
- `dashboard/`: Astro-based web UI, served via Workers Assets (Assets isolated from large rulesets).
- `compiled/`: Ruleset sync source for Cloudflare R2.

## ⚙️ Development

```bash
# Install dependencies (Node 25 + pnpm 10)
pnpm install

# Sync upstream rules
go run ./cmd/sync/

# Compile rules locally (Requires Go Stable)
go run ./cmd/compile/
```

## 📅 Maintenance Schedule

- **Automated Sync**: Every day at **07:28 SGT** and **17:16 SGT**.
- **Smart Update**: Build/Deploy process only triggers when upstream rules or MMDB data have changed.

## 🤝 Credits

- [xream/geoip](https://github.com/xream/geoip) — GeoIP & ASN source.
- [Loyalsoldier/v2ray-rules-dat](https://github.com/Loyalsoldier/v2ray-rules-dat) & [MetaCubeX/meta-rules-dat](https://github.com/MetaCubeX/meta-rules-dat).
- [Blackmatrix7/ios_rule_script](https://github.com/blackmatrix7/ios_rule_script).
- Powered by **Cloudflare Workers** & **R2 Storage**.
