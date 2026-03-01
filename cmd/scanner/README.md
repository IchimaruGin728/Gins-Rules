# Gins-Rules Scanner

Cloudflare Worker for automated official documentation scanning and AI-assisted domain classification.

## Features
- **Official Doc Sweeper**: Uses Cloudflare Browser Rendering to crawl Docker and Apple documentation.
- **Change Detection**: Stores scan results in KV to detect upstream changes.
- **Auto-Trigger**: Triggers GitHub Actions via `repository_dispatch` when changes are detected.
- **AI Classification**: Endpoint for classifying unknown domains using `llama-3-8b-instruct`.

## Setup

### 1. Variables and Bindings
Create a KV namespace named `RULES_KV`.

Update `wrangler.toml` with:
- `RULES_KV` namespace ID.
- `GITHUB_USER` and `GITHUB_REPO`.

### 2. Secrets
Set your GitHub Personal Access Token:
```bash
wrangler secret put GITHUB_TOKEN
```

### 3. Deploy
```bash
npm install
wrangler deploy
```

## API Endpoints
- `GET /scan`: Manually trigger a scan.
- `GET /classify?domain=example.com`: Get an AI-suggested category for a domain.
