/// <reference types="@cloudflare/workers-types" />

// ============================================================
// Gins-Rules Worker
// - /ruleset/:file  — Merged subscription feed
// - /health      — Health check
// - /workflow/build-complete — Trigger from GitHub Actions
// - Queue consumer — Telegram/Discord notifications
// ============================================================

interface Env {
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
  // Config
  PAGES_DOMAIN: string; // e.g. "rules.ichimarugin728.dev"
}

interface NotifyMessage {
  text: string;
  telegram: boolean;
  discord: boolean;
}

interface BuildStats {
  services: number;
  rules: number;
  formats: number;
  ipRules: number;
  asnFiles: number;
  timestamp: string;
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

    // Routes
    if (path === '/' || path === '/health') {
      return json({ status: 'ok', service: 'gins-rules', timestamp: Date.now() });
    }

    if (path.startsWith('/ruleset/')) {
      return handleFeed(path, url, env);
    }

    if (path === '/workflow/build-complete' && request.method === 'POST') {
      return handleBuildComplete(request, env);
    }

    return new Response('Not Found', { status: 404 });
  },

  // ============================================================
  // Queue Consumer — deliver notifications
  // ============================================================
  async queue(batch: MessageBatch<NotifyMessage>, env: Env): Promise<void> {
    for (const msg of batch.messages) {
      const { text, telegram, discord } = msg.body;

      const combinedText = text.replace('---SPLIT---', '\n\n').trim();

      const results = await Promise.allSettled([
        telegram ? sendTelegram(env, combinedText) : Promise.resolve(),
        discord ? sendDiscord(env, combinedText) : Promise.resolve(),
      ]);

      // Log any failures
      for (const r of results) {
        if (r.status === 'rejected') {
          console.error('Notification failed:', r.reason);
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

async function handleFeed(path: string, url: URL, env: Env): Promise<Response> {
  const fileName = path.replace('/ruleset/', '');
  const dotIdx = fileName.lastIndexOf('.');
  if (dotIdx === -1) return new Response('Invalid filename', { status: 400 });

  const type = fileName.slice(0, dotIdx);
  const ext = fileName.slice(dotIdx + 1);
  const pagesDomain = url.searchParams.get('domain') ?? env.PAGES_DOMAIN ?? 'rules.ichimarugin728.dev';

  if (!['proxy', 'direct', 'reject', 'ip'].includes(type)) {
    return new Response('Invalid category. Use: proxy, direct, reject, ip', { status: 400 });
  }

  // Map extension to directory name
  const formatMap: Record<string, string> = {
    list: 'text',
    yaml: 'egern',
    srs: 'singbox',
    mrs: 'mihomo',
  };
  const dir = formatMap[ext] ?? ext;
  
  // Use the pre-compiled *-total.* file for maximum performance
  const targetFile = `${type}-total.${ext}`;
  const targetUrl = `https://${pagesDomain}/${dir}/${type}/${targetFile}`;

  // Use 302 Found to point directly to the static file on Pages
  return Response.redirect(targetUrl, 302);
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

  const stats: BuildStats = await request.json();

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

  const domainRules = stats.rules - (stats.ipRules || 0);
  const prompt = `你是一个网络基础设施与路由专家。请按照以下严格格式撰写双语构建报告。

要求：
- 必须输出两段文字，中间用 "---SPLIT---" 分隔。
- 第一段（中文）：必须包含 "${stats.services} 服务运行平稳"、"${domainRules} 条域名规则及 ${stats.ipRules} 条 IP 记录已更新"、"一切尽在掌握 🛠️"。
- 第二段（英文）：对应中文的技术化翻译，保持极客感。
- 每段结尾：必须加上 (Updated ${date})。
- 拒绝任何其他多余的解释、前言或总结词。

示例输出：
${stats.services} 服务运行平稳，${domainRules} 条域名规则及 ${stats.ipRules} 条 IP 记录已更新，一切尽在掌握 🛠️。 (Updated ${date})
---SPLIT---
${stats.services} services online, ${domainRules} domain rules and ${stats.ipRules} IP records synchronization complete. Engineering status: OK 🛠️. (Updated ${date})

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

  const url = `https://api.telegram.org/bot${env.TELEGRAM_BOT_TOKEN}/sendMessage`;
  const resp = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      chat_id: env.TELEGRAM_CHAT_ID,
      text,
      parse_mode: 'Markdown',
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
