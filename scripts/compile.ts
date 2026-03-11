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

function mergeRules(a: Rules, b: Rules): Rules {
  return {
    domainSuffix: [...a.domainSuffix, ...b.domainSuffix],
    domain: [...a.domain, ...b.domain],
    domainKeyword: [...a.domainKeyword, ...b.domainKeyword],
    domainRegex: [...a.domainRegex, ...b.domainRegex],
    ipCidr: [...a.ipCidr, ...b.ipCidr],
  };
}

function dedup(rules: Rules): Rules {
  return {
    domainSuffix: [...new Set(rules.domainSuffix)],
    domain: [...new Set(rules.domain)],
    domainKeyword: [...new Set(rules.domainKeyword)],
    domainRegex: [...new Set(rules.domainRegex)],
    ipCidr: [...new Set(rules.ipCidr)],
  };
}

function compileSingboxJson(rules: Rules): object {
  const ruleObj: Record<string, string[]> = {};
  if (rules.domain.length) ruleObj.domain = rules.domain;
  if (rules.domainSuffix.length) ruleObj.domain_suffix = rules.domainSuffix;
  if (rules.domainKeyword.length) ruleObj.domain_keyword = rules.domainKeyword;
  if (rules.domainRegex.length) ruleObj.domain_regex = rules.domainRegex;
  if (rules.ipCidr.length) ruleObj.ip_cidr = rules.ipCidr;
  return { version: 2, rules: [ruleObj] };
}

function compileText(rules: Rules): string {
  const lines: string[] = [];
  for (const d of rules.domain) lines.push(`full:${d}`);
  for (const s of rules.domainSuffix) lines.push(s);
  for (const k of rules.domainKeyword) lines.push(`keyword:${k}`);
  for (const r of rules.domainRegex) lines.push(`regexp:${r}`);
  for (const ip of rules.ipCidr) lines.push(ip);
  return lines.join('\n') + '\n';
}

function compileMihomo(rules: Rules): string {
  const lines: string[] = [];
  for (const d of rules.domain) lines.push(`DOMAIN,${d}`);
  for (const s of rules.domainSuffix) lines.push(`DOMAIN-SUFFIX,${s}`);
  for (const k of rules.domainKeyword) lines.push(`DOMAIN-KEYWORD,${k}`);
  for (const ip of rules.ipCidr) lines.push(ip.includes(':') ? `IP-CIDR6,${ip}` : `IP-CIDR,${ip}`);
  return lines.join('\n') + '\n';
}

// Check if sing-box is available for .srs compilation
function hasSingbox(): boolean {
  try {
    execSync('sing-box version', { stdio: 'pipe' });
    return true;
  } catch {
    return false;
  }
}

async function main() {
  const root = findRoot();
  const compiledDir = path.join(root, 'compiled');
  const formats = ['singbox', 'mihomo', 'text'];
  const categories = ['proxy', 'direct', 'reject', 'ip'];
  const canCompileSrs = hasSingbox();

  console.log('============================================================');
  console.log('  Gins-Rules Compiler (TS/Bun)');
  console.log(`  sing-box SRS: ${canCompileSrs ? '✅ available' : '⚠️ not found, outputting JSON only'}`);
  console.log('============================================================');

  // Create all directories
  for (const f of formats) {
    for (const c of categories) {
      await fs.ensureDir(path.join(compiledDir, f, c));
    }
  }

  for (const category of categories) {
    const localDir = path.join(root, 'source', category);
    const upstreamDir = path.join(root, 'source', 'upstream', category);
    const ruleNames = new Set<string>();

    [localDir, upstreamDir].forEach((d: string) => {
      if (fs.existsSync(d)) {
        fs.readdirSync(d).filter((f: string) => f.endsWith('.txt')).forEach((f: string) => ruleNames.add(path.parse(f).name));
      }
    });

    for (const name of Array.from(ruleNames).sort()) {
      let rules: Rules = { domainSuffix: [], domain: [], domainKeyword: [], domainRegex: [], ipCidr: [] };

      const localFile = path.join(localDir, name + '.txt');
      const upstreamFile = path.join(upstreamDir, name + '.txt');
      
      if (fs.existsSync(localFile)) rules = mergeRules(rules, parseSource(localFile));
      if (fs.existsSync(upstreamFile)) rules = mergeRules(rules, parseSource(upstreamFile));
      rules = dedup(rules);

      const totalRules = rules.domain.length + rules.domainSuffix.length + rules.domainKeyword.length + rules.ipCidr.length;
      if (totalRules === 0) continue;

      // --- sing-box: write JSON then compile to .srs ---
      const sbDir = path.join(compiledDir, 'singbox', category);
      const jsonPath = path.join(sbDir, name + '.json');
      const srsPath = path.join(sbDir, name + '.srs');
      
      await fs.writeJson(jsonPath, compileSingboxJson(rules));

      if (canCompileSrs) {
        try {
          execSync(`sing-box rule-set compile --output "${srsPath}" "${jsonPath}"`, { stdio: 'pipe' });
          // Remove the source JSON — only keep the .srs binary
          await fs.remove(jsonPath);
        } catch (err: any) {
          console.log(`  ⚠️ [${category}] ${name}: SRS compile failed, keeping JSON`);
        }
      }

      // --- text format ---
      await fs.writeFile(path.join(compiledDir, 'text', category, name + '.txt'), compileText(rules));

      // --- mihomo / clash format ---
      await fs.writeFile(path.join(compiledDir, 'mihomo', category, name + '.txt'), compileMihomo(rules));

      console.log(`  [${category}] ${name}: ${totalRules} rules`);
    }
  }

  // Generate manifests
  for (const f of formats) {
    for (const c of categories) {
      const d = path.join(compiledDir, f, c);
      if (fs.existsSync(d)) {
        const files = fs.readdirSync(d).filter((file: string) => !fs.statSync(path.join(d, file)).isDirectory() && file !== 'manifest.json');
        await fs.writeJson(path.join(d, 'manifest.json'), files, { spaces: 2 });
      }
    }
  }

  console.log('\n  Compile complete!');
  console.log('============================================================');
}

main();
