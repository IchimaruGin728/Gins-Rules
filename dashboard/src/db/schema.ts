import { integer, sqliteTable, text } from 'drizzle-orm/sqlite-core';

export const rules = sqliteTable('rules', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  name: text('name').notNull(),
  category: text('category').notNull(), // proxy, direct, reject, ip
  ruleCount: integer('rule_count').notNull(),
  lastUpdate: integer('last_update', { mode: 'timestamp' }).notNull(),
  description: text('description'),
});

export const domains = sqliteTable('domains', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  domain: text('domain').notNull().unique(),
  ruleId: integer('rule_id').references(() => rules.id),
  category: text('category'), // AI classification result
});

export const classifications = sqliteTable('classifications', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  domain: text('domain').notNull(),
  category: text('category').notNull(),
  confidence: integer('confidence').notNull().default(90),
  model: text('model').notNull().default('llama-4-scout-17b'),
  timestamp: integer('timestamp', { mode: 'timestamp' }).notNull(),
  status: text('status').notNull().default('completed'), // completed, pending, failed
});

export const scrapeTargets = sqliteTable('scrape_targets', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  name: text('name').notNull(),
  url: text('url').notNull(),
  type: text('type').notNull().default('html'), // html, json
  enabled: integer('enabled', { mode: 'boolean' }).notNull().default(true),
  lastRun: integer('last_run', { mode: 'timestamp' }),
  createdAt: integer('created_at', { mode: 'timestamp' }).notNull(),
});
