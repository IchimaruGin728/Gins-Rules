# Gins-Rules

Comprehensive, self-maintained proxy rule list repository with fine-grained per-service categorization.

## Features
- **86+ Services**: Including Apple, Microsoft, Google, OpenAI, TikTok, etc.
- **Multi-Format Support**: 
    - **sing-box**: `.srs` (binary), `.json`
    - **Mihomo (Clash Meta) / Stash**: `.mrs` (binary), `.yaml`
    - **Surge / Loon / Shadowrocket**: `.list` (Surge format)
    - **QuantumultX**: `.list` (Native format)
    - **Egern**: `.yaml` (Native format)
- **High Performance**: Pre-compiled binary rules for modern clients.
- **Automated**: Integrated with Cloudflare Workers (Crawl/AI) and GitHub Actions.
- **Global Distribution**: Hosted on Cloudflare Pages for maximum speed and stability.

## Subscription

### 1. Merged Subscription (Dynamic via Worker)
Use this to subscribe to **all** services in a category with a single link. Replace `worker.example.com` and `pages.example.com` with your domains:

| Category | Format | URL Pattern |
|----------|--------|-------------|
| Proxy All | List | `https://worker.example.com/sub/proxy.list?domain=pages.example.com` |
| Direct All | List | `https://worker.example.com/sub/direct.list?domain=pages.example.com` |
| Reject All | List | `https://worker.example.com/sub/reject.list?domain=pages.example.com` |

### 2. Individual Service (Static via Pages)
Replace `pages.example.com` with your Pages domain:

| Client | Format | URL Pattern |
|--------|--------|-------------|
| sing-box | SRS | `https://pages.example.com/singbox/proxy/apple.srs` |
| Mihomo | MRS | `https://pages.example.com/mihomo/proxy/apple.mrs` |
| Surge / Loon | List | `https://pages.example.com/text/proxy/apple.list` |
| QuantumultX | List | `https://pages.example.com/quanx/proxy/apple.list` |
| Egern | YAML | `https://pages.example.com/egern/proxy/apple.yaml` |

## Repository Structure
- `source/`: Plain text core rule list (no comments).
- `cmd/compile/`: Go compiler for multi-format output (generates category subfolders & manifests).
- `cmd/scanner/`: CF Worker for automated scanning, AI classification, and rule merging.
- `compiled/`: Auto-generated rules for distribution (hierarchical structure).

## Development
```bash
# Compile rules locally
go run ./cmd/compile/
```

## Credits
- Official Enterprise Documentation (Apple, Microsoft, Docker, etc.)
- Original Inspiration: Blackmatrix7 (Reference only)
