import type { APIRoute } from 'astro';
import { eq, like, or } from 'drizzle-orm';
import { Hono } from 'hono';
import { getDb } from '../../db';
import { domains, rules } from '../../db/schema';

const app = new Hono<{ Bindings: { DB: D1Database } }>().basePath('/api');

// Search domains or rules
app.get('/search', async (c) => {
  const query = c.req.query('q');
  if (!query) return c.json({ results: [] });

  const db = getDb(c.env.DB);

  // Search in both domains and rules
  const domainResults = await db
    .select()
    .from(domains)
    .where(like(domains.domain, `%${query}%`))
    .limit(10);

  const ruleResults = await db
    .select()
    .from(rules)
    .where(or(like(rules.name, `%${query}%`), like(rules.category, `%${query}%`)))
    .limit(10);

  return c.json({
    domains: domainResults,
    rules: ruleResults,
  });
});

// Stats endpoint
app.get('/stats', async (c) => {
  const db = getDb(c.env.DB);
  const allRules = await db.select().from(rules);
  const totalRulesCount = allRules.reduce((acc, r) => acc + r.ruleCount, 0);

  return c.json({
    serviceCount: allRules.length,
    ruleCount: totalRulesCount,
    formats: 5,
  });
});

export const ALL: APIRoute = (context) => app.fetch(context.request, context.locals.runtime.env);
