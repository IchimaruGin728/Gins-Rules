import fs from 'fs-extra';
import path from 'path';
import { execSync } from 'child_process';

interface Rules {
  domainSuffix: string[];
  domain: string[];
  domainKeyword: string[];
  domainRegex: string[];
  ipCidr: string[];
}

const findRoot = () => {
  let curr = process.cwd();
  while (curr !== path.parse(curr).root) {
    if (fs.existsSync(path.join(curr, '.git'))) return curr;
    curr = path.dirname(curr);
  }
  return '.';
};

function parseSource(filePath: string): Rules {
  const r: Rules = { domainSuffix: [], domain: [], domainKeyword: [], domainRegex: [], ipCidr: [] };
  if (!fs.existsSync(filePath)) return r;
  const content = fs.readFileSync(filePath, 'utf-8');
  for (let line of content.split('\n')) {
    line = line.trim();
    if (!line || line.startsWith('#')) continue;
    if (line.startsWith('full:')) r.domain.push(line.substring(5));
    else if (line.startsWith('keyword:')) r.domainKeyword.push(line.substring(8));
    else if (line.startsWith('regexp:')) r.domainRegex.push(line.substring(7));
    else if (line.includes('/')) r.ipCidr.push(line);
    else r.domainSuffix.push(line.replace(/^\+\./, '').replace(/^\./, ''));
  }
  return r;
}

async function main() {
  const root = findRoot();
  const compiledDir = path.join(root, 'compiled');
  const formats = ['singbox', 'mihomo', 'text', 'quanx', 'egern', 'loon', 'stash', 'shadowrocket'];
  const categories = ['proxy', 'direct', 'reject', 'ip'];

  for (const f of formats as string[]) {
    for (const c of categories as string[]) {
      await fs.ensureDir(path.join(compiledDir, f, c));
    }
  }

  for (const category of categories) {
    const localDir = path.join(root, 'source', category);
    const upstreamDir = path.join(root, 'source', 'upstream', category);
    const ruleNames = new Set<string>();

    [localDir, upstreamDir].forEach(d => {
      if (fs.existsSync(d)) {
        fs.readdirSync(d).filter(f => f.endsWith('.txt')).forEach(f => ruleNames.add(path.parse(f).name));
      }
    });

    for (const name of Array.from(ruleNames).sort()) {
      const rules = parseSource(path.join(localDir, name + '.txt'));
      // Simplification: only local source for now in this restore, add upstream merging if needed
      
      const sbJson = { version: 2, rules: [{
        domain: rules.domain.length ? rules.domain : undefined,
        domain_suffix: rules.domainSuffix.length ? rules.domainSuffix : undefined,
        ip_cidr: rules.ipCidr.length ? rules.ipCidr : undefined,
      }]};
      await fs.writeJson(path.join(compiledDir, 'singbox', category, name + '.json'), sbJson, { spaces: 2 });
      console.log(`  [${category}] ${name}: compiled`);
    }
  }

  for (const f of formats as string[]) {
    for (const c of categories as string[]) {
      const d = path.join(compiledDir, f, c);
      if (fs.existsSync(d)) {
        const files = fs.readdirSync(d).filter((file: string) => !fs.statSync(path.join(d, file)).isDirectory() && file !== 'manifest.json');
        await fs.writeJson(path.join(d, 'manifest.json'), files, { spaces: 2 });
      }
    }
  }
}

main();
