/// <reference types="@cloudflare/workers-types" />
import { Hono } from 'hono';
import puppeteer from '@cloudflare/puppeteer';

type Bindings = {
  MYBROWSER: Fetcher;
  RULES_KV: KVNamespace;
  AI: Ai;
  GITHUB_TOKEN: string;
  GITHUB_USER: string;
  GITHUB_REPO: string;
};

const app = new Hono<{ Bindings: Bindings }>();

// AI Classification using Llama-4-Scout (MoE)
async function classifyDomain(env: Bindings, domain: string): Promise<string> {
  const prompt = `Task: Classify technical domain into network routing category.
Domain: ${domain}
Categories:
- proxy: International/Restricted services (e.g. Google, OpenAI)
- direct: Local/Official infrastructure (e.g. apple.com, alicdn.com)
- reject: Tracking/Advertising/Privacy-invasive
Response format: Output EXACTLY one word from the categories above.`;

  try {
    const response = await env.AI.run('@cf/meta/llama-4-scout-17b-16e-instruct', {
      messages: [{ role: 'user', content: prompt }],
    });

    return response?.response?.toLowerCase()?.trim() ?? 'proxy';
  } catch (err) {
    console.error('AI classification failed:', err);
    return 'proxy'; // Safe fallback
  }
}

// Main crawl logic
async function crawlOfficialDocs(env: Bindings): Promise<Record<string, string[]>> {
  const browser = await puppeteer.launch(env.MYBROWSER);
  try {
    const page = await browser.newPage();
    page.setDefaultNavigationTimeout(30_000); // 30s timeout

    // 1. Docker Allowlist
    await page.goto('https://docs.docker.com/desktop/setup/allow-list/', {
      waitUntil: 'domcontentloaded',
    });
    const dockerDomains = (await page.evaluate(`
      Array.from(document.querySelectorAll('a'))
        .map(a => a.innerText)
        .filter(text => text.startsWith('https://'))
        .map(url => url.replace('https://', ''))
        .filter((v, i, arr) => arr.indexOf(v) === i)
    `)) as string[];

    return { docker: dockerDomains };
  } catch (err) {
    console.error('Crawling failed:', err);
    return { docker: [] };
  } finally {
    await browser.close();
  }
}

// Trigger GitHub Actions repository dispatch
async function triggerGitHub(env: Bindings): Promise<boolean> {
  const url = `https://api.github.com/repos/${env.GITHUB_USER}/${env.GITHUB_REPO}/dispatches`;
  const response = await fetch(url, {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${env.GITHUB_TOKEN}`,
      Accept: 'application/vnd.github.v3+json',
      'User-Agent': 'Gins-Rules-Scanner',
    },
    body: JSON.stringify({ event_type: 'scrape_update' }),
  });
  return response.ok;
}

app.get('/scan', async (c) => {
  const results = await crawlOfficialDocs(c.env);
  const oldData = await c.env.RULES_KV.get('last_scan_result');
  const newData = JSON.stringify(results);

  if (newData !== oldData) {
    await c.env.RULES_KV.put('last_scan_result', newData);
    const success = await triggerGitHub(c.env);
    return c.json({ status: 'changed', triggered: success, data: results });
  }

  return c.json({ status: 'no_change' });
});

app.get('/classify', async (c) => {
  const domain = c.req.query('domain');
  if (!domain) return c.text('Missing domain', 400);
  const category = await classifyDomain(c.env, domain);
  return c.json({ domain, category });
});

/**
 * Dynamic Rule Aggregator
 * Usage: GET /sub/proxy.list?domain=your-pages-url.com
 */
app.get('/sub/:file', async (c) => {
  const fileName = c.req.param('file');
  const dotIdx = fileName.lastIndexOf('.');
  if (dotIdx === -1) return c.text('Invalid filename', 400);

  const type = fileName.slice(0, dotIdx);
  const ext = fileName.slice(dotIdx + 1);
  const pagesDomain = c.req.query('domain') ?? 'rules.ichimarugin.com';

  if (!['proxy', 'direct', 'reject', 'ip'].includes(type)) {
    return c.text('Invalid rule category', 400);
  }

  const formatMap: Record<string, string> = {
    list: 'text',
    yaml: 'egern',
    srs: 'singbox',
    mrs: 'mihomo',
  };
  const dir = formatMap[ext] ?? ext;
  const baseUrl = `https://${pagesDomain}/${dir}/${type}`;

  try {
    const manifestResp = await fetch(`${baseUrl}/manifest.json`);
    if (!manifestResp.ok) return c.text('Manifest not found', 404);

    const files: string[] = await manifestResp.json();
    const results = await Promise.all(
      files.map((f) => fetch(`${baseUrl}/${f}`).then((r: Response) => r.text())),
    );

    let merged = results.join('\n');

    // Simple deduplication for .list files
    if (ext === 'list') {
      const lines = merged
        .split('\n')
        .map((l: string) => l.trim())
        .filter((l: string) => l && !l.startsWith('#'))
        .filter((v: string, i: number, a: string[]) => a.indexOf(v) === i);
      merged = lines.join('\n');
    }

    return c.text(merged);
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err);
    return c.text(`Error merging rules: ${msg}`, 500);
  }
});

// Scheduled trigger
export default {
  async scheduled(_event: ScheduledEvent, env: Bindings, ctx: ExecutionContext) {
    ctx.waitUntil(
      crawlOfficialDocs(env).then(async (results) => {
        const oldData = await env.RULES_KV.get('last_scan_result');
        const newData = JSON.stringify(results);
        if (newData !== oldData) {
          await env.RULES_KV.put('last_scan_result', newData);
          await triggerGitHub(env);
        }
      }),
    );
  },
  fetch: app.fetch,
};
