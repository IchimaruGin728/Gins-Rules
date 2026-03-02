package main

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"
)


type Rules struct {
	DomainSuffix  []string
	Domain        []string
	DomainKeyword []string
	DomainRegex   []string
	IPCIDR        []string
}


type SingBoxRuleSet struct {
	Version int              `json:"version"`
	Rules   []SingBoxRule    `json:"rules"`
}

type SingBoxRule struct {
	DomainSuffix  []string `json:"domain_suffix,omitempty"`
	Domain        []string `json:"domain,omitempty"`
	DomainKeyword []string `json:"domain_keyword,omitempty"`
	DomainRegex   []string `json:"domain_regex,omitempty"`
	IPCIDR        []string `json:"ip_cidr,omitempty"`
}


type Stats struct {
	Files int
	Rules int
	SRS   int
	MRS   int
}

func main() {
	root := findRoot()
	compiledDir := filepath.Join(root, "compiled")

	singboxDir := filepath.Join(compiledDir, "singbox")
	mihomoDir := filepath.Join(compiledDir, "mihomo")
	textDir := filepath.Join(compiledDir, "text")
	quanxDir := filepath.Join(compiledDir, "quanx")
	egernDir := filepath.Join(compiledDir, "egern")
	loonDir := filepath.Join(compiledDir, "loon")
	stashDir := filepath.Join(compiledDir, "stash")
	shadowrocketDir := filepath.Join(compiledDir, "shadowrocket")

	fmt.Println("============================================================")
	fmt.Println("  Gins-Rules Compiler (Go)")
	fmt.Println("============================================================")

	binDir := filepath.Join(root, "bin")
	mihomoPath := findBinary("mihomo", binDir)
	singboxPath := findBinary("sing-box", binDir)

	fmt.Printf("\n  sing-box: %s (%s)\n", boolIcon(singboxPath != ""), singboxPath)
	fmt.Printf("  mihomo:   %s (%s)\n\n", boolIcon(mihomoPath != ""), mihomoPath)

	hasSingBox := singboxPath != ""
	hasMihomo := mihomoPath != ""


	for _, dir := range []string{singboxDir, mihomoDir, textDir, quanxDir, egernDir, loonDir, stashDir, shadowrocketDir} {
		os.RemoveAll(dir)
		os.MkdirAll(dir, 0o755)
		for _, cat := range []string{"proxy", "direct", "reject", "ip"} {
			os.MkdirAll(filepath.Join(dir, cat), 0o755)
			if dir == mihomoDir || dir == stashDir {
				os.MkdirAll(filepath.Join(dir, cat, "yaml"), 0o755)
			}
		}
	}

	categories := []string{"proxy", "direct", "reject", "ip"}
	stats := Stats{}

	for _, category := range categories {
		ruleNames := make(map[string]bool)
		
		localDir := filepath.Join(root, "source", category)
		upstreamDir := filepath.Join(root, "source", "upstream", category)

		for _, d := range []string{localDir, upstreamDir} {
			if _, err := os.Stat(d); err == nil {
				for _, f := range listTxtFiles(d) {
					ruleNames[strings.TrimSuffix(filepath.Base(f), ".txt")] = true
				}
			}
		}

		isIP := category == "ip"

		var sortedNames []string
		for name := range ruleNames {
			sortedNames = append(sortedNames, name)
		}
		sort.Strings(sortedNames)

		for _, name := range sortedNames {
			localPath := filepath.Join(localDir, name+".txt")
			upstreamPath := filepath.Join(upstreamDir, name+".txt")

			var rules Rules
			if _, err := os.Stat(localPath); err == nil {
				rules = parseSource(localPath)
			}
			if _, err := os.Stat(upstreamPath); err == nil {
				uRules := parseSource(upstreamPath)
				rules = mergeRules(rules, uRules)
			}

			total := len(rules.DomainSuffix) + len(rules.Domain) +
				len(rules.DomainKeyword) + len(rules.DomainRegex) + len(rules.IPCIDR)
			if total == 0 {
				continue
			}

			jsonPath := compileSingBoxJSON(name, rules, filepath.Join(singboxDir, category))
			srsOK := false
			if hasSingBox {
				srsOK = compileSingBoxSRS(jsonPath, filepath.Join(singboxDir, category), singboxPath)
			}

			yamlPath := compileMihomoYAML(name, rules, filepath.Join(mihomoDir, category, "yaml"), isIP)
			behavior := "domain"
			if isIP {
				behavior = "ipcidr"
			}
			mrsOK := false
			if hasMihomo {
				mrsOK = compileMihomoMRS(yamlPath, filepath.Join(mihomoDir, category), behavior, mihomoPath)
			}

			stashYAML := compileMihomoYAML(name, rules, filepath.Join(stashDir, category, "yaml"), isIP)
			if hasMihomo && mrsOK {
				compileMihomoMRS(stashYAML, filepath.Join(stashDir, category), behavior, mihomoPath)
			}

			count := compileTextList(name, rules, filepath.Join(textDir, category), isIP)
			compileQuanXList(name, rules, filepath.Join(quanxDir, category), isIP)
			compileEgernYAML(name, rules, filepath.Join(egernDir, category)) 
			compileLoonList(name, rules, filepath.Join(loonDir, category))
			compileLoonList(name, rules, filepath.Join(shadowrocketDir, category))

			srsIcon := "·"
			if srsOK {
				srsIcon = "✓"
			}
			mrsIcon := "·"
			if mrsOK {
				mrsIcon = "✓"
			}

			fmt.Printf("  [%-6s] %-20s %3d rules  srs:%s  mrs:%s\n",
				category, name, count, srsIcon, mrsIcon)

			stats.Files++
			stats.Rules += count
			if srsOK {
				stats.SRS++
			}
			if mrsOK {
				stats.MRS++
			}
		}
	}


	for _, formatDir := range []string{"singbox", "mihomo", "text", "quanx", "egern", "loon", "stash", "shadowrocket"} {
		for _, cat := range categories {
			dir := filepath.Join(compiledDir, formatDir, cat)
			entries, _ := os.ReadDir(dir)
			var fileList []string
			for _, e := range entries {
				if !e.IsDir() && e.Name() != "manifest.json" {
					fileList = append(fileList, e.Name())
				}
			}
			if len(fileList) > 0 {
				mContent, _ := json.MarshalIndent(fileList, "", "  ")
				os.WriteFile(filepath.Join(dir, "manifest.json"), mContent, 0o644)
			}
		}
	}

	fmt.Printf("\n============================================================\n")
	fmt.Printf("  Done! %d files, %d total rules\n", stats.Files, stats.Rules)
	fmt.Printf("  SRS: %d/%d  MRS: %d/%d\n", stats.SRS, stats.Files, stats.MRS, stats.Files)
	fmt.Printf("  Output: %s\n", compiledDir)
	fmt.Println("============================================================")
}

func findRoot() string {
	exe, _ := os.Executable()
	dir := filepath.Dir(exe)


	for d := dir; d != "/"; d = filepath.Dir(d) {
		if _, err := os.Stat(filepath.Join(d, "go.mod")); err == nil {
			return d
		}
	}


	wd, _ := os.Getwd()
	for d := wd; d != "/"; d = filepath.Dir(d) {
		if _, err := os.Stat(filepath.Join(d, "go.mod")); err == nil {
			return d
		}
	}

	fmt.Fprintln(os.Stderr, "Error: cannot find project root (go.mod)")
	os.Exit(1)
	return ""
}

func findBinary(name, binDir string) string {
	local := filepath.Join(binDir, name)
	if _, err := os.Stat(local); err == nil {
		return local
	}
	p, err := exec.LookPath(name)
	if err == nil {
		return p
	}
	return ""
}

func boolIcon(b bool) string {
	if b {
		return "✓"
	}
	return "✗"
}

func listTxtFiles(dir string) []string {
	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil
	}
	var files []string
	for _, e := range entries {
		if !e.IsDir() && strings.HasSuffix(e.Name(), ".txt") {
			files = append(files, filepath.Join(dir, e.Name()))
		}
	}
	sort.Strings(files)
	return files
}

func parseSource(path string) Rules {
	var r Rules
	data, err := os.ReadFile(path)
	if err != nil {
		return r
	}
	for _, line := range strings.Split(string(data), "\n") {
		line = strings.TrimSpace(line)
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		switch {
		case strings.HasPrefix(line, "full:"):
			r.Domain = append(r.Domain, line[5:])
		case strings.HasPrefix(line, "keyword:"):
			r.DomainKeyword = append(r.DomainKeyword, line[8:])
		case strings.HasPrefix(line, "regexp:"):
			r.DomainRegex = append(r.DomainRegex, line[7:])
		case strings.Contains(line, "/"):
			r.IPCIDR = append(r.IPCIDR, line)
		default:
			// Sanitize domain: remove +. and leading dots
			domain := line
			domain = strings.TrimPrefix(domain, "+.")
			domain = strings.TrimPrefix(domain, ".")
			r.DomainSuffix = append(r.DomainSuffix, domain)
		}
	}
	return r
}

func mergeRules(a, b Rules) Rules {
	a.DomainSuffix = append(a.DomainSuffix, b.DomainSuffix...)
	a.Domain = append(a.Domain, b.Domain...)
	a.DomainKeyword = append(a.DomainKeyword, b.DomainKeyword...)
	a.DomainRegex = append(a.DomainRegex, b.DomainRegex...)
	a.IPCIDR = append(a.IPCIDR, b.IPCIDR...)

	a.DomainSuffix = unique(a.DomainSuffix)
	a.Domain = unique(a.Domain)
	a.DomainKeyword = unique(a.DomainKeyword)
	a.DomainRegex = unique(a.DomainRegex)
	a.IPCIDR = unique(a.IPCIDR)
	return a
}

func unique(slice []string) []string {
	keys := make(map[string]bool)
	list := []string{}
	for _, entry := range slice {
		if _, value := keys[entry]; !value {
			keys[entry] = true
			list = append(list, entry)
		}
	}
	sort.Strings(list)
	return list
}

func compileSingBoxJSON(name string, rules Rules, outDir string) string {
	rule := SingBoxRule{
		DomainSuffix:  rules.DomainSuffix,
		Domain:        rules.Domain,
		DomainKeyword: rules.DomainKeyword,
		DomainRegex:   rules.DomainRegex,
		IPCIDR:        rules.IPCIDR,
	}

	rs := SingBoxRuleSet{
		Version: 2,
		Rules:   []SingBoxRule{rule},
	}

	jsonPath := filepath.Join(outDir, name+".json")
	data, _ := json.MarshalIndent(rs, "", "  ")
	os.WriteFile(jsonPath, append(data, '\n'), 0o644)
	return jsonPath
}

func compileSingBoxSRS(jsonPath, outDir, singboxPath string) bool {
	name := strings.TrimSuffix(filepath.Base(jsonPath), ".json")
	srsPath := filepath.Join(outDir, name+".srs")
	cmd := exec.Command(singboxPath, "rule-set", "compile", jsonPath, "-o", srsPath)
	output, err := cmd.CombinedOutput()
	if err != nil {
		fmt.Fprintf(os.Stderr, "  [ERROR] sing-box compile %s: %v\n  Output: %s\n", name, err, string(output))
		return false
	}
	return true
}

func compileMihomoYAML(name string, rules Rules, outDir string, isIP bool) string {
	var lines []string
	lines = append(lines, fmt.Sprintf("# Gins-Rules: %s", name))
	lines = append(lines, "# Auto-generated, do not edit")
	lines = append(lines, "")
	lines = append(lines, "payload:")

	if isIP {
		for _, cidr := range rules.IPCIDR {
			lines = append(lines, fmt.Sprintf("  - '%s'", cidr))
		}
	} else {
		var domains []string
		domains = append(domains, rules.DomainSuffix...)
		domains = append(domains, rules.Domain...)
		
		uniqueDomains := make(map[string]bool)
		for _, d := range domains {
			// Skip domains with too many dots to avoid Mihomo panic
			if strings.Count(d, ".") > 5 {
				continue
			}
			if !uniqueDomains[d] {
				uniqueDomains[d] = true
				lines = append(lines, fmt.Sprintf("  - '%s'", d))
			}
		}
	}

	yamlPath := filepath.Join(outDir, name+".yaml")
	os.WriteFile(yamlPath, []byte(strings.Join(lines, "\n")+"\n"), 0o644)
	return yamlPath
}

func compileMihomoMRS(yamlPath, outDir, behavior, mihomoPath string) bool {
	name := strings.TrimSuffix(filepath.Base(yamlPath), ".yaml")
	mrsPath := filepath.Join(outDir, name+".mrs")
	cmd := exec.Command(mihomoPath, "convert-ruleset", behavior, "yaml", yamlPath, mrsPath)
	output, err := cmd.CombinedOutput()
	if err != nil {
		fmt.Fprintf(os.Stderr, "  [ERROR] mihomo compile %s: %v\n  Output: %s\n", name, err, string(output))
		return false
	}
	return true
}

func compileTextList(name string, rules Rules, outDir string, isIP bool) int {
	var lines []string

	for _, d := range rules.DomainSuffix {
		lines = append(lines, fmt.Sprintf("DOMAIN-SUFFIX,%s", d))
	}
	for _, d := range rules.Domain {
		lines = append(lines, fmt.Sprintf("DOMAIN,%s", d))
	}
	for _, d := range rules.DomainKeyword {
		lines = append(lines, fmt.Sprintf("DOMAIN-KEYWORD,%s", d))
	}
	for _, cidr := range rules.IPCIDR {
		prefix := "IP-CIDR"
		if strings.Contains(cidr, ":") {
			prefix = "IP-CIDR6"
		}
		lines = append(lines, fmt.Sprintf("%s,%s", prefix, cidr))
	}

	suffix := ".list"
	if isIP {
		suffix = ".ip.list"
	}

	header := fmt.Sprintf("# Gins-Rules: %s\n# Auto-generated, do not edit\n# Total: %d rules\n\n", name, len(lines))
	content := header + strings.Join(lines, "\n") + "\n"

	listPath := filepath.Join(outDir, name+suffix)
	os.WriteFile(listPath, []byte(content), 0o644)
	return len(lines)
}

func compileQuanXList(name string, rules Rules, outDir string, isIP bool) {
	var lines []string

	for _, d := range rules.DomainSuffix {
		lines = append(lines, fmt.Sprintf("host-suffix, %s", d))
	}
	for _, d := range rules.Domain {
		lines = append(lines, fmt.Sprintf("host, %s", d))
	}
	for _, d := range rules.DomainKeyword {
		lines = append(lines, fmt.Sprintf("host-keyword, %s", d))
	}
	for _, cidr := range rules.IPCIDR {
		if strings.Contains(cidr, ":") {
			lines = append(lines, fmt.Sprintf("ip6-cidr, %s", cidr))
		} else {
			lines = append(lines, fmt.Sprintf("ip-cidr, %s", cidr))
		}
	}

	suffix := ".list"
	if isIP {
		suffix = ".ip.list"
	}

	content := strings.Join(lines, "\n") + "\n"
	listPath := filepath.Join(outDir, name+suffix)
	os.WriteFile(listPath, []byte(content), 0o644)
}

func compileEgernYAML(name string, rules Rules, outDir string) {
	var lines []string
	lines = append(lines, "# Gins-Rules: "+name)
	lines = append(lines, "# Optimized Egern Rule Set")
	lines = append(lines, "")

	if len(rules.DomainSuffix) > 0 {
		lines = append(lines, "domain_suffix_set:")
		for _, d := range rules.DomainSuffix {
			lines = append(lines, fmt.Sprintf("  - %s", d))
		}
	}
	if len(rules.Domain) > 0 {
		lines = append(lines, "domain_set:")
		for _, d := range rules.Domain {
			lines = append(lines, fmt.Sprintf("  - %s", d))
		}
	}
	if len(rules.DomainKeyword) > 0 {
		lines = append(lines, "domain_keyword_set:")
		for _, d := range rules.DomainKeyword {
			lines = append(lines, fmt.Sprintf("  - %s", d))
		}
	}

	var v4, v6 []string
	for _, cidr := range rules.IPCIDR {
		if strings.Contains(cidr, ":") {
			v6 = append(v6, cidr)
		} else {
			v4 = append(v4, cidr)
		}
	}
	if len(v4) > 0 {
		lines = append(lines, "ip_cidr_set:")
		for _, c := range v4 {
			lines = append(lines, fmt.Sprintf("  - %s", c))
		}
	}
	if len(v6) > 0 {
		lines = append(lines, "ip_cidr6_set:")
		for _, c := range v6 {
			lines = append(lines, fmt.Sprintf("  - %s", c))
		}
	}

	content := strings.Join(lines, "\n") + "\n"
	os.WriteFile(filepath.Join(outDir, name+".yaml"), []byte(content), 0o644)
}

func compileLoonList(name string, rules Rules, outDir string) {
	var lines []string

	for _, d := range rules.DomainSuffix {
		lines = append(lines, fmt.Sprintf("DOMAIN-SUFFIX,%s", d))
	}
	for _, d := range rules.Domain {
		lines = append(lines, fmt.Sprintf("DOMAIN,%s", d))
	}
	for _, d := range rules.DomainKeyword {
		lines = append(lines, fmt.Sprintf("DOMAIN-KEYWORD,%s", d))
	}
	for _, cidr := range rules.IPCIDR {
		prefix := "IP-CIDR"
		if strings.Contains(cidr, ":") {
			prefix = "IP-CIDR6"
		}
		lines = append(lines, fmt.Sprintf("%s,%s", prefix, cidr))
	}

	suffix := ".list"
	content := strings.Join(lines, "\n") + "\n"
	listPath := filepath.Join(outDir, name+suffix)
	os.WriteFile(listPath, []byte(content), 0o644)
}
