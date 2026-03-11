import fs from 'fs-extra';
import path from 'path';
import { execSync } from 'child_process';

// ── Types ──

interface Rules {
  domainSuffix: string[];
  domain: string[];
  domainKeyword: string[];
  domainRegex: string[];
  ipCidr: string[];
}

// ── Utilities ──

const findRoot = (): string => {
  let curr = process.cwd();
  while (curr !== path.parse(curr).root) {
    if (fs.existsSync(path.join(curr, '.git'))) return curr;
    curr = path.dirname(curr);
  }
  return '.';
};

function hasBinary(name: string): string {
  try {
    const p = execSync(`which ${name}`, { stdio: 'pipe' }).toString().trim();
    return p;
  } catch {
    return '';
  }
}

function parseSource(filePath: string): Rules {
  const r: Rules = { domainSuffix: [], domain: [], domainKeyword: [], domainRegex: [], ipCidr: [] };
  if (!fs.existsSync(filePath)) return r;
  const content = fs.readFileSync(filePath, 'utf-8');
  for (let line of content.split('\n')) {
    line = line.trim();
    if (!line || line.startsWith('#') || line.startsWith(';') || line.startsWith('//')) continue;

    // Try to parse as comma-separated rule (e.g. DOMAIN-SUFFIX,example.com)
    const parts = line.split(',');
    if (parts.length >= 2) {
      const type = parts[0].trim().toUpperCase();
      const val = parts[1].trim();
      switch (type) {
        case 'DOMAIN-SUFFIX': case 'HOST-SUFFIX': r.domainSuffix.push(val); continue;
        case 'DOMAIN': case 'HOST': r.domain.push(val); continue;
        case 'DOMAIN-KEYWORD': case 'HOST-KEYWORD': r.domainKeyword.push(val); continue;
        case 'IP-CIDR': case 'IP-CIDR6': r.ipCidr.push(val); continue;
      }
    }

    // Internal format
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

function totalRules(r: Rules): number {
  return r.domainSuffix.length + r.domain.length + r.domainKeyword.length + r.domainRegex.length + r.ipCidr.length;
}

// ── Format Compilers ──

// sing-box JSON → .srs
function compileSingboxJSON(name: string, rules: Rules, outDir: string): string {
  const ruleObj: Record<string, string[]> = {};
  if (rules.domainSuffix.length) ruleObj.domain_suffix = rules.domainSuffix;
  if (rules.domain.length) ruleObj.domain = rules.domain;
  if (rules.domainKeyword.length) ruleObj.domain_keyword = rules.domainKeyword;
  if (rules.domainRegex.length) ruleObj.domain_regex = rules.domainRegex;
  if (rules.ipCidr.length) ruleObj.ip_cidr = rules.ipCidr;

  const jsonPath = path.join(outDir, name + '.json');
  fs.writeJsonSync(jsonPath, { version: 2, rules: [ruleObj] }, { spaces: 2 });
  return jsonPath;
}

function compileSingboxSRS(jsonPath: string, outDir: string, singboxPath: string): boolean {
  const name = path.parse(jsonPath).name;
  const srsPath = path.join(outDir, name + '.srs');
  try {
    execSync(`"${singboxPath}" rule-set compile "${jsonPath}" -o "${srsPath}"`, { stdio: 'pipe' });
    return true;
  } catch (err: any) {
    console.error(`  [ERROR] sing-box compile ${name}: ${err.message}`);
    return false;
  }
}

// mihomo YAML → .mrs
function compileMihomoYAML(name: string, rules: Rules, outDir: string, isIP: boolean): string {
  const lines: string[] = [];
  lines.push(`# Gins-Rules: ${name}`);
  lines.push('# Auto-generated, do not edit');
  lines.push('');
  lines.push('payload:');

  if (isIP) {
    for (const cidr of rules.ipCidr) lines.push(`  - '${cidr}'`);
  } else {
    const seen = new Set<string>();
    for (const d of [...rules.domainSuffix, ...rules.domain]) {
      if (d.split('.').length > 6) continue; // Skip domains with too many dots
      if (!seen.has(d)) {
        seen.add(d);
        lines.push(`  - '${d}'`);
      }
    }
  }

  const yamlPath = path.join(outDir, name + '.yaml');
  fs.writeFileSync(yamlPath, lines.join('\n') + '\n');
  return yamlPath;
}

function compileMihomoMRS(yamlPath: string, outDir: string, behavior: string, mihomoPath: string): boolean {
  const name = path.parse(yamlPath).name;
  const mrsPath = path.join(outDir, name + '.mrs');
  try {
    execSync(`"${mihomoPath}" convert-ruleset ${behavior} yaml "${yamlPath}" "${mrsPath}"`, { stdio: 'pipe' });
    return true;
  } catch (err: any) {
    console.error(`  [ERROR] mihomo compile ${name}: ${err.message}`);
    return false;
  }
}

// text .list format
function compileTextList(name: string, rules: Rules, outDir: string, isIP: boolean): number {
  const lines: string[] = [];
  for (const d of rules.domainSuffix) lines.push(`DOMAIN-SUFFIX,${d}`);
  for (const d of rules.domain) lines.push(`DOMAIN,${d}`);
  for (const d of rules.domainKeyword) lines.push(`DOMAIN-KEYWORD,${d}`);
  for (const cidr of rules.ipCidr) lines.push(`${cidr.includes(':') ? 'IP-CIDR6' : 'IP-CIDR'},${cidr}`);

  const suffix = isIP ? '.ip.list' : '.list';
  const header = `# Gins-Rules: ${name}\n# Auto-generated, do not edit\n# Total: ${lines.length} rules\n\n`;
  fs.writeFileSync(path.join(outDir, name + suffix), header + lines.join('\n') + '\n');
  return lines.length;
}

// QuantumultX format
function compileQuanXList(name: string, rules: Rules, outDir: string, isIP: boolean, category: string): void {
  const policy = category === 'direct' ? 'Direct' : category === 'reject' ? 'Reject' : 'Proxy';
  const lines: string[] = [];
  for (const d of rules.domainSuffix) lines.push(`host-suffix,${d},${policy}`);
  for (const d of rules.domain) lines.push(`host,${d},${policy}`);
  for (const d of rules.domainKeyword) lines.push(`host-keyword,${d},${policy}`);
  for (const cidr of rules.ipCidr) lines.push(`${cidr.includes(':') ? 'ip6-cidr' : 'ip-cidr'},${cidr},${policy}`);

  const suffix = isIP ? '.ip.list' : '.list';
  fs.writeFileSync(path.join(outDir, name + suffix), lines.join('\n') + '\n');
}

// Egern YAML format
function compileEgernYAML(name: string, rules: Rules, outDir: string): void {
  const lines: string[] = [];
  lines.push('# Gins-Rules: ' + name);
  lines.push('# Optimized Egern Rule Set');
  lines.push('');

  if (rules.domainSuffix.length) {
    lines.push('domain_suffix_set:');
    for (const d of rules.domainSuffix) lines.push(`  - ${d}`);
  }
  if (rules.domain.length) {
    lines.push('domain_set:');
    for (const d of rules.domain) lines.push(`  - ${d}`);
  }
  if (rules.domainKeyword.length) {
    lines.push('domain_keyword_set:');
    for (const d of rules.domainKeyword) lines.push(`  - ${d}`);
  }

  const v4 = rules.ipCidr.filter((c: string) => !c.includes(':'));
  const v6 = rules.ipCidr.filter((c: string) => c.includes(':'));
  if (v4.length) {
    lines.push('ip_cidr_set:');
    for (const c of v4) lines.push(`  - ${c}`);
  }
  if (v6.length) {
    lines.push('ip_cidr6_set:');
    for (const c of v6) lines.push(`  - ${c}`);
  }

  fs.writeFileSync(path.join(outDir, name + '.yaml'), lines.join('\n') + '\n');
}

// Loon / Shadowrocket format (same as text but no header)
function compileLoonList(name: string, rules: Rules, outDir: string): void {
  const lines: string[] = [];
  for (const d of rules.domainSuffix) lines.push(`DOMAIN-SUFFIX,${d}`);
  for (const d of rules.domain) lines.push(`DOMAIN,${d}`);
  for (const d of rules.domainKeyword) lines.push(`DOMAIN-KEYWORD,${d}`);
  for (const cidr of rules.ipCidr) lines.push(`${cidr.includes(':') ? 'IP-CIDR6' : 'IP-CIDR'},${cidr}`);

  fs.writeFileSync(path.join(outDir, name + '.list'), lines.join('\n') + '\n');
}

// ── Main ──

async function main() {
  const root = findRoot();
  const compiledDir = path.join(root, 'compiled');
  const categories = ['proxy', 'direct', 'reject', 'ip'];
  const formatDirs = ['singbox', 'mihomo', 'text', 'quanx', 'egern', 'loon', 'stash', 'shadowrocket'];

  const singboxPath = hasBinary('sing-box');
  const mihomoPath = hasBinary('mihomo');

  console.log('============================================================');
  console.log('  Gins-Rules Compiler (TS/Bun)');
  console.log(`  sing-box: ${singboxPath ? '✅' : '⚠️  not found'} ${singboxPath}`);
  console.log(`  mihomo:   ${mihomoPath ? '✅' : '⚠️  not found'} ${mihomoPath}`);
  console.log('============================================================');

  // Create all directories
  for (const f of formatDirs) {
    for (const c of categories) {
      await fs.ensureDir(path.join(compiledDir, f, c));
      if (f === 'mihomo' || f === 'stash') {
        await fs.ensureDir(path.join(compiledDir, f, c, 'yaml'));
      }
    }
  }

  let totalFiles = 0, totalRulesCount = 0, srsCount = 0, mrsCount = 0;

  for (const category of categories) {
    const localDir = path.join(root, 'source', category);
    const upstreamDir = path.join(root, 'source', 'upstream', category);
    const isIP = category === 'ip';
    const ruleNames = new Set<string>();

    for (const d of [localDir, upstreamDir]) {
      if (fs.existsSync(d)) {
        fs.readdirSync(d).filter((f: string) => f.endsWith('.txt')).forEach((f: string) => ruleNames.add(path.parse(f).name));
      }
    }

    for (const name of Array.from(ruleNames).sort()) {
      let rules: Rules = { domainSuffix: [], domain: [], domainKeyword: [], domainRegex: [], ipCidr: [] };
      
      const localFile = path.join(localDir, name + '.txt');
      const upstreamFile = path.join(upstreamDir, name + '.txt');
      if (fs.existsSync(localFile)) rules = mergeRules(rules, parseSource(localFile));
      if (fs.existsSync(upstreamFile)) rules = mergeRules(rules, parseSource(upstreamFile));
      rules = dedup(rules);

      const total = totalRules(rules);
      if (total === 0) continue;

      // ── sing-box: JSON → .srs ──
      const jsonPath = compileSingboxJSON(name, rules, path.join(compiledDir, 'singbox', category));
      let srsOK = false;
      if (singboxPath) {
        srsOK = compileSingboxSRS(jsonPath, path.join(compiledDir, 'singbox', category), singboxPath);
      }

      // ── mihomo: YAML → .mrs ──
      const mihomoYaml = compileMihomoYAML(name, rules, path.join(compiledDir, 'mihomo', category, 'yaml'), isIP);
      const behavior = isIP ? 'ipcidr' : 'domain';
      let mrsOK = false;
      if (mihomoPath) {
        const isEmpty = isIP ? rules.ipCidr.length === 0 : (rules.domain.length === 0 && rules.domainSuffix.length === 0);
        if (!isEmpty) {
          mrsOK = compileMihomoMRS(mihomoYaml, path.join(compiledDir, 'mihomo', category), behavior, mihomoPath);
        }
      }

      // ── stash: same as mihomo (YAML → .mrs) ──
      const stashYaml = compileMihomoYAML(name, rules, path.join(compiledDir, 'stash', category, 'yaml'), isIP);
      if (mihomoPath && mrsOK) {
        compileMihomoMRS(stashYaml, path.join(compiledDir, 'stash', category), behavior, mihomoPath);
      }

      // ── text ──
      const count = compileTextList(name, rules, path.join(compiledDir, 'text', category), isIP);

      // ── QuantumultX ──
      compileQuanXList(name, rules, path.join(compiledDir, 'quanx', category), isIP, category);

      // ── Egern ──
      compileEgernYAML(name, rules, path.join(compiledDir, 'egern', category));

      // ── Loon ──
      compileLoonList(name, rules, path.join(compiledDir, 'loon', category));

      // ── Shadowrocket ──
      compileLoonList(name, rules, path.join(compiledDir, 'shadowrocket', category));

      console.log(`  [%-6s] %-20s %3d rules  srs:${srsOK ? '✓' : '·'}  mrs:${mrsOK ? '✓' : '·'}`.replace('%-6s', category.padEnd(6)).replace('%-20s', name.padEnd(20)).replace('%3d', String(count).padStart(3)));

      totalFiles++;
      totalRulesCount += count;
      if (srsOK) srsCount++;
      if (mrsOK) mrsCount++;
    }
  }

  // Generate manifests
  for (const f of formatDirs) {
    for (const c of categories) {
      const d = path.join(compiledDir, f, c);
      if (fs.existsSync(d)) {
        const files = fs.readdirSync(d).filter((file: string) => !fs.statSync(path.join(d, file)).isDirectory() && file !== 'manifest.json');
        if (files.length > 0) {
          await fs.writeJson(path.join(d, 'manifest.json'), files, { spaces: 2 });
        }
      }
    }
  }

  console.log(`\n  ✅ ${totalFiles} files, ${totalRulesCount} rules, ${srsCount} SRS, ${mrsCount} MRS`);
  console.log('============================================================');
}

main();
