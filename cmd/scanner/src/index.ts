/// <reference types="@cloudflare/workers-types" />
import { Hono } from 'hono';
import { cors } from 'hono/cors';
import puppeteer from '@cloudflare/puppeteer';

type Bindings = {
  MYBROWSER: Fetcher;
  RULES_KV: KVNamespace;
  AI: Ai;
  GITHUB_TOKEN: string;
  GITHUB_USER: string;
  GITHUB_REPO: string;
  GINS_INTERNAL_TOKEN: string;
  DB: D1Database;
};

const app = new Hono<{ Bindings: Bindings }>();

// Root and Health Check
app.get('/', (c) => c.text('Gins Rules Scanner - Operational', 200));
app.get('/health', (c) => c.json({ status: 'ok', timestamp: Date.now() }));

// Enable CORS for the dashboard
app.use('*', async (c, next) => {
  const corsMiddleware = cors({
    origin: ['https://dash.ichimarugin728.dev', 'http://localhost:4321'],
    allowHeaders: ['X-Gins-Auth', 'Content-Type'],
    allowMethods: ['GET', 'POST', 'OPTIONS'],
    maxAge: 86400,
  });
  return corsMiddleware(c, next);
});

// Internal Auth Middleware for /scan and /classify
app.use('/scan', async (c, next) => {
  const auth = c.req.header('X-Gins-Auth');
  if (auth !== c.env.GINS_INTERNAL_TOKEN) return c.text('Unauthorized', 401);
  await next();
});

app.use('/classify', async (c, next) => {
  const auth = c.req.header('X-Gins-Auth');
  if (auth !== c.env.GINS_INTERNAL_TOKEN) return c.text('Unauthorized', 401);
  await next();
});


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

    const category = response?.response?.toLowerCase()?.trim() ?? 'proxy';

    // Log to D1 for AI Monitor
    try {
      await env.DB.prepare(
        'INSERT INTO classifications (domain, category, confidence, model, timestamp) VALUES (?, ?, ?, ?, ?)'
      ).bind(domain, category, 95, '@cf/meta/llama-4-scout-17b-16e-instruct', Date.now()).run();
    } catch (d1Err) {
      console.error('Failed to log classification to D1:', d1Err);
    }

    return category;
  } catch (err) {
    console.error('AI classification failed:', err);
    return 'proxy'; // Safe fallback
  }
}

// Puppeteer Budget: max launches per day (Paid plan = 10hr/mo ≈ 600min)
// Each launch ≈ 30-60s, so 5/day = ~150/mo = ~2.5hr/mo (safe margin)
const PUPPETEER_DAILY_LIMIT = 5;

async function canUsePuppeteer(kv: KVNamespace): Promise<boolean> {
  const today = new Date().toISOString().slice(0, 10);
  const key = `browser_usage_${today}`;
  const count = parseInt(await kv.get(key) ?? '0');
  return count < PUPPETEER_DAILY_LIMIT;
}

async function trackPuppeteerUsage(kv: KVNamespace): Promise<void> {
  const today = new Date().toISOString().slice(0, 10);
  const key = `browser_usage_${today}`;
  const count = parseInt(await kv.get(key) ?? '0');
  await kv.put(key, String(count + 1), { expirationTtl: 172800 }); // Auto-expire in 48h
}

async function crawlTarget(env: Bindings, target: { name: string; url: string; type: string }): Promise<string[]> {
  const domains: Set<string> = new Set();
  console.log(`Processing target [${target.name}] (${target.type}): ${target.url}`);

  try {
    if (target.type === 'json') {
      // Logic for Official Vendor JSONs (AWS, Azure, etc.)
      const resp = await fetch(target.url);
      if (resp.ok) {
        const data = await resp.json() as any;
        const text = JSON.stringify(data);
        // Robust regex for finding domains in unknown JSON structures
        const matches = text.match(/[a-z0-9.-]+\.(?:com|net|org|io|gov|edu|ai|co|dev|me)\b/gi);
        matches?.forEach(d => domains.add(d.toLowerCase()));
      }
    } else {
      // Phase 1: High-Speed Fetch + HTMLRewriter
      const response = await fetch(target.url);
      if (response.ok) {
        await new HTMLRewriter()
          .on('a[href]', {
            element(el) {
              const href = el.getAttribute('href');
              if (href?.startsWith('https://')) {
                try {
                  const domain = new URL(href).hostname;
                  if (domain && domain.includes('.')) domains.add(domain.toLowerCase());
                } catch {}
              }
            },
          })
          .on('code', {
            text(text) {
              const content = text.text.trim();
              if (content.match(/^[a-z0-9.-]+\.[a-z]{2,}$/i)) {
                domains.add(content.toLowerCase());
              }
            }
          })
          .transform(response)
          .arrayBuffer();
      }

      // Phase 2: Puppeteer Fallback (Budget-Controlled)
      if (domains.size === 0) {
        const allowed = await canUsePuppeteer(env.RULES_KV);
        if (!allowed) {
          console.log(`[QUOTA] Puppeteer daily limit reached (${PUPPETEER_DAILY_LIMIT}/day). Skipping fallback for ${target.name}`);
        } else {
          console.log(`[PUPPETEER] Launching browser for ${target.name}...`);
          await trackPuppeteerUsage(env.RULES_KV);
          const browser = await puppeteer.launch(env.MYBROWSER);
          try {
            const page = await browser.newPage();
            await page.goto(target.url, { waitUntil: 'domcontentloaded', timeout: 30000 });
            const dynamicDomains = (await page.evaluate(`
              Array.from(document.querySelectorAll('a'))
                .map(a => a.innerText)
                .filter(text => text.startsWith('https://'))
                .map(url => url.replace('https://', ''))
            `)) as string[];
            dynamicDomains.forEach(d => domains.add(d.toLowerCase()));
          } finally {
            await browser.close();
          }
        }
      }
    }
  } catch (err) {
    console.error(`Target [${target.name}] failed:`, err);
  }

  return Array.from(domains);
}


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
  // 1. Fetch enabled targets from D1
  const targetsRaw = await c.env.DB.prepare('SELECT * FROM scrape_targets WHERE enabled = 1').all();
  const targets = targetsRaw.results as unknown as any[];
  
  if (targets.length === 0) {
    return c.json({ status: 'no_targets', message: 'No enabled scrape targets in D1' });
  }

  // 2. Process all targets
  const allDomains: Record<string, string[]> = {};
  for (const t of targets) {
    const found = await crawlTarget(c.env, t);
    if (found.length > 0) {
      allDomains[t.name.toLowerCase().replace(/\s+/g, '_')] = found;
      // Update last_run
      await c.env.DB.prepare('UPDATE scrape_targets SET last_run = ? WHERE id = ?')
        .bind(Date.now(), t.id).run();
    }
  }

  // 3. Compare and Trigger
  const oldData = await c.env.RULES_KV.get('last_scan_result');
  const newData = JSON.stringify(allDomains);

  if (newData !== oldData) {
    await c.env.RULES_KV.put('last_scan_result', newData);
    const success = await triggerGitHub(c.env);
    return c.json({ status: 'changed', triggered: success, targets_processed: targets.length });
  }

  return c.json({ status: 'no_change', targets_processed: targets.length });
});

app.get('/classify', async (c) => {
  const domain = c.req.query('domain');
  if (!domain) return c.text('Missing domain', 400);
  const category = await classifyDomain(c.env, domain);
  return c.json({ domain, category });
});


app.get('/sub/:file', async (c) => {
  const fileName = c.req.param('file');
  const dotIdx = fileName.lastIndexOf('.');
  if (dotIdx === -1) return c.text('Invalid filename', 400);

  const type = fileName.slice(0, dotIdx);
  const ext = fileName.slice(dotIdx + 1);
  const pagesDomain = c.req.query('domain') ?? 'rules.ichimarugin728.dev';

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


export default {
  async scheduled(_event: ScheduledEvent, env: Bindings, ctx: ExecutionContext) {
    ctx.waitUntil(
      Promise.all([
        (async () => {
          const targetsRaw = await env.DB.prepare('SELECT * FROM scrape_targets WHERE enabled = 1').all();
          const targets = targetsRaw.results as unknown as any[];
          if (targets.length === 0) return;

          const allDomains: Record<string, string[]> = {};
          for (const t of targets) {
            const found = await crawlTarget(env, t);
            if (found.length > 0) {
              allDomains[t.name.toLowerCase().replace(/\s+/g, '_')] = found;
              await env.DB.prepare('UPDATE scrape_targets SET last_run = ? WHERE id = ?')
                .bind(Date.now(), t.id).run();
            }
          }

          const oldData = await env.RULES_KV.get('last_scan_result');
          const newData = JSON.stringify(allDomains);
          if (newData !== oldData) {
            await env.RULES_KV.put('last_scan_result', newData);
            await triggerGitHub(env);
          }
        })(),
        // Clean up old AI logs: Keep only the most recent 1000
        env.DB.prepare(`
          DELETE FROM classifications 
          WHERE id NOT IN (
            SELECT id FROM classifications 
            ORDER BY timestamp DESC 
            LIMIT 1000
          )
        `).run().catch(e => console.error('Retention cleanup failed:', e))
      ])
    );
  },
  fetch: app.fetch,
};
