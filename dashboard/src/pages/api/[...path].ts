import type { APIRoute } from 'astro';
import { desc, like, or, eq } from 'drizzle-orm';
import { Hono } from 'hono';
import { getDb } from '../../db';
import { classifications, domains, rules, scrapeTargets } from '../../db/schema';

type Env = {
  DB: D1Database;
  CF_API_TOKEN: string;
  CF_ACCOUNT_ID: string;
  GINS_INTERNAL_TOKEN: string;
};

const app = new Hono<{ Bindings: Env }>().basePath('/api');

app.get('/ai/logs', async (c) => {
  const db = getDb(c.env.DB);
  const logs = await db
    .select()
    .from(classifications)
    .orderBy(desc(classifications.timestamp))
    .limit(20);
  return c.json(logs);
});

app.post('/ai/trigger', async (c) => {
  const resp = await fetch('https://scanner.ichimarugin728.dev/scan', {
    headers: { 'X-Gins-Auth': c.env.GINS_INTERNAL_TOKEN },
  });
  const data = await resp.json();
  return c.json(data);
});

app.get('/search', async (c) => {
  const query = c.req.query('q');
  if (!query) return c.json({ domains: [], rules: [] });

  const db = getDb(c.env.DB);

  // 1. Search local DB
  const [domainResults, ruleResults] = await Promise.all([
    db.select().from(domains).where(like(domains.domain, `%${query}%`)).limit(10),
    db.select().from(rules).where(or(like(rules.name, `%${query}%`), like(rules.category, `%${query}%`))).limit(10)
  ]);

  // 2. Elite Fallback: If it looks like a domain and no direct match found, trigger AI Scan
  if (domainResults.length === 0 && 
      query.includes('.') && 
      !query.includes(' ') && 
      query.length > 3) {
    try {
      const scannerResp = await fetch(`https://scanner.ichimarugin728.dev/classify?domain=${query}`, {
        headers: { 'X-Gins-Auth': c.env.GINS_INTERNAL_TOKEN }
      });
      if (scannerResp.ok) {
        const aiResult = (await scannerResp.json()) as { domain: string; category: string };
        return c.json({ 
          domains: [{ domain: aiResult.domain, category: aiResult.category, isAiGenerated: true }], 
          rules: ruleResults 
        });
      }
    } catch (e) {
      console.error('AI Search fallback failed:', e);
    }
  }

  return c.json({ domains: domainResults, rules: ruleResults });
});

app.get('/stats', async (c) => {
  const db = getDb(c.env.DB);
  const allRules = await db.select().from(rules);
  const totalRulesCount = allRules.reduce((acc, r) => acc + r.ruleCount, 0);
  return c.json({ serviceCount: allRules.length, ruleCount: totalRulesCount, formats: 5 });
});

// Target Management API
app.get('/targets', async (c) => {
  const db = getDb(c.env.DB);
  const targets = await db.select().from(scrapeTargets);
  return c.json(targets);
});

app.post('/targets', async (c) => {
  const { name, url, type } = await c.req.json();
  const db = getDb(c.env.DB);
  const result = await db.insert(scrapeTargets).values({
    name,
    url,
    type: type || 'html',
    createdAt: new Date(),
    enabled: true
  }).returning();
  return c.json(result[0]);
});

app.put('/targets/:id', async (c) => {
  const id = parseInt(c.req.param('id'));
  const { enabled } = await c.req.json();
  const db = getDb(c.env.DB);
  await db.update(scrapeTargets).set({ enabled }).where(eq(scrapeTargets.id, id));
  return c.json({ ok: true });
});

app.delete('/targets/:id', async (c) => {
  const id = parseInt(c.req.param('id'));
  const db = getDb(c.env.DB);
  await db.delete(scrapeTargets).where(eq(scrapeTargets.id, id));
  return c.json({ ok: true });
});

app.get('/status', async (c) => {
  const { CF_API_TOKEN, CF_ACCOUNT_ID } = c.env;
  const headers = {
    Authorization: `Bearer ${CF_API_TOKEN}`,
    'Content-Type': 'application/json',
  };

  const now = new Date();
  const since = new Date(now.getTime() - 24 * 60 * 60 * 1000).toISOString();
  const until = now.toISOString();
  const sinceDate = since.slice(0, 10);
  const untilDate = until.slice(0, 10);

  const gqlQuery = `query {
    viewer {
      accounts(filter: { accountTag: "${CF_ACCOUNT_ID}" }) {
        workersInvocationsAdaptive(
          filter: { scriptName: "gins-rules-scanner", datetime_geq: "${since}", datetime_leq: "${until}" }
          limit: 1
        ) {
          sum { requests errors subrequests }
          quantiles { cpuTimeP50 cpuTimeP99 }
        }
        workersAnalyticsEngineAdaptive: workersInvocationsAdaptive(
          filter: { datetime_geq: "${since}", datetime_leq: "${until}" }
          limit: 1
        ) {
          sum { requests }
        }
        aiGatewayRequestsAdaptive(
          filter: { datetimeHour_geq: "${since}", datetimeHour_leq: "${until}" }
          limit: 1
        ) {
          sum { requests tokens }
        }
        httpRequestsAdaptive: httpRequests1dGroups(
          filter: { date_geq: "${sinceDate}", date_leq: "${untilDate}" }
          limit: 1
        ) {
          sum { requests bytes }
        }
      }
    }
  }`;

  const [analyticsRes, healthRes] = await Promise.all([
    fetch('https://api.cloudflare.com/client/v4/graphql', {
      method: 'POST',
      headers,
      body: JSON.stringify({ query: gqlQuery }),
    }).then((r: Response) => r.json()).catch(() => null),

    fetch('https://scanner.ichimarugin728.dev/classify?domain=cloudflare.com', {
      headers: { 'X-Gins-Auth': c.env.GINS_INTERNAL_TOKEN },
      signal: AbortSignal.timeout(5000),
    }).then(async (r: Response) => ({
      ok: r.ok,
      latency: 0,
      result: await r.json().catch(() => null),
    })).catch(() => ({ ok: false, latency: -1, result: null })),
  ]);

  const acct = (analyticsRes as any)?.data?.viewer?.accounts?.[0];
  const scanner = acct?.workersInvocationsAdaptive?.[0];
  const allWorkers = acct?.workersAnalyticsEngineAdaptive?.[0];
  const ai = acct?.aiGatewayRequestsAdaptive?.[0];
  const http = acct?.httpRequestsAdaptive?.[0];

  return c.json({
    scanner: {
      healthy: healthRes.ok,
      requests24h: scanner?.sum?.requests ?? 0,
      errors24h: scanner?.sum?.errors ?? 0,
      subrequests24h: scanner?.sum?.subrequests ?? 0,
      cpuP50: scanner?.quantiles?.cpuTimeP50 ?? 0,
      cpuP99: scanner?.quantiles?.cpuTimeP99 ?? 0,
    },
    ai: {
      requests24h: ai?.sum?.requests ?? 0,
      tokens24h: ai?.sum?.tokens ?? 0,
    },
    cdn: {
      requests24h: http?.sum?.requests ?? 0,
      bandwidth24h: http?.sum?.bytes ?? 0,
    },
    totalWorkerRequests: allWorkers?.sum?.requests ?? 0,
    timestamp: now.toISOString(),
  });
});

export const ALL: APIRoute = (context) => app.fetch(context.request, context.locals.runtime.env);
