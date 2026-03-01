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
