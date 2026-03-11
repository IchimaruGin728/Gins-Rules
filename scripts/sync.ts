import fs from 'fs-extra';
import path from 'path';
import axios from 'axios';

interface UpstreamSource {
  name: string;
  url: string;
  category: 'proxy' | 'direct' | 'reject';
  target: string;
  enabled: boolean;
}

const findRoot = () => {
  let curr = process.cwd();
  while (curr !== path.parse(curr).root) {
    if (fs.existsSync(path.join(curr, '.git'))) return curr;
    curr = path.dirname(curr);
  }
  return '.';
};

async function main() {
  const root = findRoot();
  const configPath = path.join(root, 'source', 'sources.json');
  const sources: UpstreamSource[] = await fs.readJson(configPath);
  const upstreamDir = path.join(root, 'source', 'upstream');

  console.log('============================================================');
  console.log('  Gins-Rules Upstream Syncer (TS/Bun)');
  console.log('============================================================');

  const mergedResults: Record<string, Record<string, string[]>> = {};

  for (const src of sources) {
    if (!src.enabled) continue;
    console.log(`  Fetching ${src.name}...`);

    try {
      const resp = await axios.get(src.url, { timeout: 30000 });
      const rules = resp.data.split('\n').filter((l: string) => l.trim() && !l.startsWith('#')).map((l: string) => l.trim());
      
      if (!mergedResults[src.category]) mergedResults[src.category] = {};
      if (!mergedResults[src.category][src.target]) mergedResults[src.category][src.target] = [];
      mergedResults[src.category][src.target].push(...rules);
    } catch (err: any) {
      console.error(`  [ERROR] ${src.name}: ${err.message}`);
    }
  }

  for (const [cat, targets] of Object.entries(mergedResults)) {
    for (const [name, rules] of Object.entries(targets)) {
      const outPath = path.join(upstreamDir, cat, `${name}.txt`);
      await fs.ensureDir(path.dirname(outPath));
      await fs.writeFile(outPath, Array.from(new Set(rules)).sort().join('\n') + '\n');
    }
  }

  console.log('  Sync complete!');
}

main();
