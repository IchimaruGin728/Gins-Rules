/// <reference types="@cloudflare/workers-types" />

// ============================================================
// Gins-Rules Worker
// - /ruleset/:file  — Merged subscription feed
// - /health      — Health check
// - /workflow/build-complete — Trigger from GitHub Actions
// - Queue consumer — Telegram/Discord notifications
// ============================================================

interface Env {
  // Workers Assets
  ASSETS: Fetcher;
  // R2 Storage
  RULES_BUCKET: R2Bucket;
  // Workflow
  BUILD_WORKFLOW: Workflow;
  // Queue
  NOTIFY_QUEUE: Queue;
  // AI
  AI: Ai;
  // Secrets (set via wrangler secret)
  TELEGRAM_BOT_TOKEN: string;
  TELEGRAM_CHAT_ID: string;
  DISCORD_WEBHOOK_URL: string;
  WORKFLOW_SECRET: string;
}

interface NotifyMessage {
  text: string;
  telegram: boolean;
  discord: boolean;
}

interface BuildStats {
  services: number;
  rules: number;
  ipRules: number;
  asnFiles: number;
  srs: number;
  mrs: number;
  categoryCounts: Record<string, number>;
  formats: number;
  timestamp: string;
}

interface BuildCompletePayload {
  metrics?: Partial<BuildStats> & { icons_total?: number };
  timestamp?: string;
}

// ============================================================
// HTTP Handler
// ============================================================

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    const path = url.pathname;

    // CORS
    if (request.method === 'OPTIONS') {
      return new Response(null, {
        headers: corsHeaders(),
      });
    }

    // 1. Ruleset Requests (Served from R2 via handleFeed)
    if (path.startsWith('/ruleset/')) {
      return handleFeed(request, path, url, env);
    }

    if (path === '/Gins-Icons.json') {
      return handleIconsCatalog(request, env);
    }

    // 2. Static Assets (Dashboard components & Canonical Rules)
    // We try to serve the exact file or a matching directory index first.
    try {
      const assetResp = await (env.ASSETS as any).fetch(request.clone());
      if (assetResp.status !== 404) return assetResp;
    } catch (e) {
      console.error('Asset fetch error:', e);
    }

    // 3. API & Internal Workflow Handlers
    if (path === '/health' || path === '/api/health') {
      return json({ status: 'ok', service: 'gins-rules', timestamp: Date.now() });
    }

    if (path === '/workflow/build-complete' && request.method === 'POST') {
      return handleBuildComplete(request, env);
    }

    // 4. Final Fallback (Serve index.html for SPA/Dashboard entry)
    return (env.ASSETS as any).fetch(new Request(new URL('/', request.url).toString(), request));
  },

  // ============================================================
  // Queue Consumer — deliver notifications
  // ============================================================
  async queue(batch: MessageBatch<NotifyMessage>, env: Env): Promise<void> {
    for (const msg of batch.messages) {
      const { text, telegram, discord } = msg.body;

      const messages = text.split('---SPLIT---').map((m) => m.trim()).filter(Boolean);

      for (const msgContent of messages) {
        const results = await Promise.allSettled([
          telegram ? sendTelegram(env, msgContent) : Promise.resolve(),
          discord ? sendDiscord(env, msgContent) : Promise.resolve(),
        ]);

        // Log any failures
        for (const r of results) {
          if (r.status === 'rejected') {
            console.error('Notification failed:', r.reason);
          }
        }
      }

      msg.ack();
    }
  },
};

// ============================================================
// Cloudflare Workflow — Orchestration
// ============================================================

import { WorkflowEntrypoint, WorkflowEvent, WorkflowStep } from 'cloudflare:workers';

export class BuildWorkflow extends WorkflowEntrypoint<Env, BuildStats> {
  async run(event: WorkflowEvent<BuildStats>, step: WorkflowStep) {
    const stats = event.payload;

    // Step 1: Generate AI Summary
    const summary = await step.do('generate-summary', async () => {
      return await generateDailySummary(this.env, stats);
    });

    // Step 2: Enqueue Notification
    await step.do('enqueue-notification', async () => {
      await this.env.NOTIFY_QUEUE.send({
        text: summary,
        telegram: true,
        discord: true,
      } satisfies NotifyMessage);
    });
  }
}

// ============================================================
// /ruleset/:file — Merged subscription
// ============================================================

async function handleFeed(request: Request, path: string, url: URL, env: Env): Promise<Response> {
  // Special handling for Xray/V2Ray binary DAT files
  if (
    path === '/ruleset/xray/geosite.dat' || 
    path === '/ruleset/xray/geoip.dat' ||
    path === '/ruleset/v2ray/geosite.dat' ||
    path === '/ruleset/v2ray/geoip.dat' ||
    path === '/ruleset/geoip.mmdb' ||
    path === '/ruleset/geoasn.mmdb'
  ) {
    // Map both /xray/ and /v2ray/ to the same R2 directory 'ruleset/xray/'
    // BUT for mmdb files at the root of /ruleset/, keep the path as is (removing leading slash)
    let key: string;
    if (path.endsWith('.mmdb')) {
        key = path.replace(/^\//, '');
    } else {
        key = path.replace(/^\/(ruleset)\/(v2ray|xray)\//, '$1/xray/').replace(/^\//, '');
    }
    
    const r2Object = await env.RULES_BUCKET.get(key);
    if (!r2Object) return new Response('Binary Asset Not Found in R2: ' + key, { status: 404 });
    const headers = new Headers();
    r2Object.writeHttpMetadata(headers);
    headers.set('etag', r2Object.httpEtag);
    headers.set('Access-Control-Allow-Origin', '*');
    headers.set('Content-Type', 'application/octet-stream');
    headers.set('Cache-Control', 'public, max-age=3600');
    return new Response(r2Object.body, { headers });
  }

  const parts = path.replace('/ruleset/', '').split('/');
  
  let app: string | null = null;
  let category: string;
  let name: string;
  let ext: string;

  if (parts.length === 1) {
    // Format: ruleset/proxy.srs (Legacy/Merged Default)
    const fileName = parts[0];
    const dotIdx = fileName.lastIndexOf('.');
    if (dotIdx === -1) return new Response('Invalid filename', { status: 400 });
    category = fileName.slice(0, dotIdx);
    name = category;
    ext = fileName.slice(dotIdx + 1);
  } else if (parts.length === 2) {
    // Format A: ruleset/proxy/google.srs (Merged Default)
    // Format B: ruleset/shadowrocket/proxy.list (New App-specific Merged)
    // Format C: ruleset/loon/proxy.lsr (Loon-specific Merged)
    // We check if parts[0] is a known app or a known category
    const apps = ['singbox', 'mihomo', 'stash', 'surge', 'quantumultx', 'quanx', 'loon', 'egern', 'shadowrocket', 'surfboard', 'exclave'];
    const categories = ['proxy', 'direct', 'reject', 'ip', 'asn', 'ai'];
    
    if (apps.includes(parts[0])) {
      app = parts[0];
      const fileName = parts[1];
      const dotIdx = fileName.lastIndexOf('.');
      if (dotIdx === -1) return new Response('Invalid filename', { status: 400 });
      category = fileName.slice(0, dotIdx);
      name = category;
      ext = fileName.slice(dotIdx + 1);
    } else {
      category = parts[0];
      const fileName = parts[1];
      const dotIdx = fileName.lastIndexOf('.');
      if (dotIdx === -1) return new Response('Invalid filename', { status: 400 });
      name = fileName.slice(0, dotIdx);
      ext = fileName.slice(dotIdx + 1);
    }
  } else if (parts.length === 3) {
    // Format: ruleset/shadowrocket/proxy/google.list
    // Format: ruleset/loon/proxy/google.lsr
    app = parts[0];
    category = parts[1];
    const fileName = parts[2];
    const dotIdx = fileName.lastIndexOf('.');
    if (dotIdx === -1) return new Response('Invalid filename', { status: 400 });
    name = fileName.slice(0, dotIdx);
    ext = fileName.slice(dotIdx + 1);
  } else {
    return new Response('Not Found', { status: 404 });
  }

  if (!['proxy', 'direct', 'reject', 'ip', 'asn', 'ai'].includes(category)) {
    return new Response('Invalid category', { status: 400 });
  }

  // App to Directory Mapping
  const appToDir: Record<string, string> = {
    'singbox': 'singbox',
    'mihomo': 'mihomo',
    'clash': 'mihomo',
    'stash': 'stash',
    'surge': 'surge',
    'quantumultx': 'quantumultx',
    'quanx': 'quantumultx',
    'loon': 'loon',
    'egern': 'egern',
    'shadowrocket': 'shadowrocket',
    'surfboard': 'surfboard',
    'surfboard_ds': 'surfboard',
    'exclave': 'exclave',
    'v2ray': 'xray',
  };

  // Extension Mapping
  const extMap: Record<string, string> = {
    'list': 'text',
    'lsr': 'loon',
    'yaml': 'egern',
    'srs': 'singbox',
    'mrs': 'mihomo',
  };

  const dir = app ? appToDir[app] : (extMap[ext] ?? ext);

  if (!dir) return new Response('Invalid app or extension', { status: 400 });

  let targetFile = `${name}.${ext}`;
  if (app === 'loon' && ext === 'list') {
    targetFile = `${name}.lsr`;
  }
  // Special correction for IP/ASN lists in text format (compiler outputs .ip.list)
  const isIPLike = category === 'ip' || category === 'asn';
  if (dir === 'text' && isIPLike && ext === 'list' && !targetFile.includes('.ip.')) {
    targetFile = targetFile.replace('.list', '.ip.list');
  }
  const assetPath = `ruleset/${dir}/${category}/${targetFile}`;
  const r2Object = await env.RULES_BUCKET.get(assetPath);

  if (!r2Object) {
    return new Response('Rule Not Found in R2', { status: 404 });
  }

  const headers = new Headers();
  r2Object.writeHttpMetadata(headers);
  headers.set('etag', r2Object.httpEtag);
  headers.set('Access-Control-Allow-Origin', '*');

  return new Response(r2Object.body, { headers });
}

async function handleIconsCatalog(request: Request, env: Env): Promise<Response> {
  const assetResp = await (env.ASSETS as any).fetch(
    new Request(new URL('/icons-catalog.json', request.url).toString(), request),
  );

  if (assetResp.status === 404) {
    return new Response('Icons catalog not found', { status: 404 });
  }

  const headers = new Headers(assetResp.headers);
  headers.set('Content-Type', 'application/json; charset=utf-8');
  headers.set('Access-Control-Allow-Origin', '*');
  return new Response(assetResp.body, {
    status: assetResp.status,
    headers,
  });
}

// ============================================================
// /workflow/build-complete — GitHub Actions trigger
// ============================================================

async function handleBuildComplete(request: Request, env: Env): Promise<Response> {
  // Verify secret
  const auth = request.headers.get('Authorization');
  if (auth !== `Bearer ${env.WORKFLOW_SECRET}`) {
    return new Response('Unauthorized', { status: 401 });
  }

  const payload = await request.json() as BuildStats | BuildCompletePayload;
  const stats = normalizeBuildStats(payload);

  // Use build timestamp as ID to deduplicate (GitHub Actions 'on push' might trigger twice)
  // Format: 20260311T120000Z
  const instanceId = `build-${stats.timestamp.replace(/[:.-]/g, '')}`;

  try {
    const instance = await env.BUILD_WORKFLOW.create({
      id: instanceId,
      params: stats,
    });

    return json({
      status: 'accepted',
      workflow_id: instance.id,
      message: 'Workflow triggered successfully',
    });
  } catch (err) {
    // If instance already exists, it's a deduplicated duplicate trigger
    if (err instanceof Error && err.message.includes('already exists')) {
      return json({
        status: 'deduplicated',
        message: 'Workflow already running for this build',
      });
    }
    throw err;
  }
}

// ============================================================
// Workers AI — Daily summary with personality
// ============================================================

async function generateDailySummary(env: Env, stats: BuildStats): Promise<string> {
  const date = new Date().toLocaleDateString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    timeZone: 'Asia/Singapore',
  });

  const domainRules = Math.max(0, stats.rules - (stats.ipRules || 0));
  const srsRate = stats.services > 0 ? ((stats.srs / stats.services) * 100).toFixed(0) : '0';
  const mrsRate = stats.services > 0 ? ((stats.mrs / stats.services) * 100).toFixed(0) : '0';

  const prompt = `你是一个网络基础设施与路由专家。请按照以下严格格式撰写双语构建报告。

要求：
- 必须输出两段文字，中间用 "---SPLIT---" 分隔。
- 第一段（中文）：标题使用 *加粗*，核心数据使用 \`行内代码\`。必须包含 "🛡️ Gins-Rules 报告"、"\`${stats.services}\` 个分流服务活跃中"、"已更新 \`${domainRules}\` 条域名规则及 \`${stats.ipRules}\` 条 IP 记录"、"二进制编译率：\`SRS ${srsRate}% / MRS ${mrsRate}%\`"。
- 第二段（英文）：对应中文的技术化翻译，保持极客感，使用相应的 Markdown 格式。
- 每段结尾：必须加上 (Updated ${date})。
- 拒绝任何其他多余的解释、前言或总结词。

示例输出：
*🛡️ Gins-Rules 报告*
✅ \`${stats.services}\` 个分流服务活跃中，已更新 \`${domainRules}\` 条域名规则及 \`${stats.ipRules}\` 条 IP 记录。二进制编译率：\`SRS ${srsRate}% / MRS ${mrsRate}%\`。 (Updated ${date})
---SPLIT---
*🛡️ Gins-Rules Report*
✅ \`${stats.services}\` routing services active. Synchronized \`${domainRules}\` domain rules and \`${stats.ipRules}\` IP records. Binary compilation: \`SRS ${srsRate}% / MRS ${mrsRate}%\`. (Updated ${date})

直接输出消息内容。`;

  try {
    const response = await env.AI.run('@cf/meta/llama-4-scout-17b-16e-instruct', {
      messages: [{ role: 'user', content: prompt }],
    });

    const text = response?.response?.trim();
    if (text && text.length > 5) return text;
  } catch (err) {
    console.error('AI summary failed:', err);
  }

  // Fallback if AI fails
  return `🛡️ Gins-Rules 日报 · ${date}\n\n✅ ${stats.services} 服务 · ${stats.rules} 规则 · ${stats.formats} 格式\n☁️ 已部署，一切正常 🏄`;
}

// ============================================================
// Notification Delivery
// ============================================================

async function sendTelegram(env: Env, text: string): Promise<void> {
  if (!env.TELEGRAM_BOT_TOKEN || !env.TELEGRAM_CHAT_ID) return;

  // Use MarkdownV2 for newer formatting, but we must escape certain characters outside of entities
  // AI generated markdown entities like *bold* are fine, but individual . or ! might need escaping in V2
  // For simplicity and compatibility with AI output, we use Markdown (V1) but with rich entities
  // because MarkdownV2 is extremely brittle without a heavy-duty escaper.
  
  const url = `https://api.telegram.org/bot${env.TELEGRAM_BOT_TOKEN}/sendMessage`;
  const resp = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      chat_id: env.TELEGRAM_CHAT_ID,
      text,
      parse_mode: 'Markdown', // Standard Markdown is more robust for AI generation
      disable_web_page_preview: true,
    }),
  });

  if (!resp.ok) {
    const body = await resp.text();
    throw new Error(`Telegram API error: ${resp.status} ${body}`);
  }
}

async function sendDiscord(env: Env, text: string): Promise<void> {
  if (!env.DISCORD_WEBHOOK_URL) return;

  const resp = await fetch(env.DISCORD_WEBHOOK_URL, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ content: text }),
  });

  if (!resp.ok) {
    const body = await resp.text();
    throw new Error(`Discord webhook error: ${resp.status} ${body}`);
  }
}

// ============================================================
// Helpers
// ============================================================

function json(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: {
      'Content-Type': 'application/json',
      ...corsHeaders(),
    },
  });
}

function corsHeaders(): Record<string, string> {
  return {
    'Access-Control-Allow-Origin': '*',
    'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
    'Access-Control-Allow-Headers': 'Content-Type, Authorization',
    'Access-Control-Max-Age': '86400',
  };
}

function normalizeBuildStats(payload: BuildStats | BuildCompletePayload): BuildStats {
  const source: Partial<BuildStats> & { icons_total?: number } =
    'metrics' in payload && payload.metrics
      ? payload.metrics
      : (payload as Partial<BuildStats> & { icons_total?: number });

  return {
    services: Number(source.services ?? 0),
    rules: Number(source.rules ?? 0),
    ipRules: Number(source.ipRules ?? 0),
    asnFiles: Number(source.asnFiles ?? 0),
    srs: Number(source.srs ?? 0),
    mrs: Number(source.mrs ?? 0),
    categoryCounts: source.categoryCounts ?? {},
    formats: Number(source.formats ?? 0),
    timestamp: String(source.timestamp ?? ('timestamp' in payload ? payload.timestamp : '') ?? ''),
  };
}
