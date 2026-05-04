package main

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
)

type Rules struct {
	DomainSuffix  []string
	Domain        []string
	DomainKeyword []string
	DomainRegex   []string
	IPCIDR        []string
	ProcessName   []string
	UserAgent     []string
}

type SingBoxRuleSet struct {
	Version int           `json:"version"`
	Rules   []SingBoxRule `json:"rules"`
}

type SingBoxRule struct {
	DomainSuffix  []string `json:"domain_suffix,omitempty"`
	Domain        []string `json:"domain,omitempty"`
	DomainKeyword []string `json:"domain_keyword,omitempty"`
	DomainRegex   []string `json:"domain_regex,omitempty"`
	IPCIDR        []string `json:"ip_cidr,omitempty"`
}

type Stats struct {
	Services       int            `json:"services"`
	Rules          int            `json:"rules"`
	IPRules        int            `json:"ipRules"`
	ASNFiles       int            `json:"asnFiles"`
	SRS            int            `json:"srs"`
	MRS            int            `json:"mrs"`
	CategoryCounts map[string]int `json:"categoryCounts"`
	Formats        int            `json:"formats"`
}

type mihomoRuleMode struct {
	isIP     bool
	behavior string
	isEmpty  bool
}

var aiRuleNames = []string{
	"ai-other",
	"apple-intelligence",
	"claude",
	"copilot",
	"gemini",
	"openai",
}

func main() {
	root := findRoot()
	compiledDir := filepath.Join(root, "compiled")
	rulesetDir := filepath.Join(compiledDir, "ruleset")

	singboxDir := filepath.Join(rulesetDir, "singbox")
	mihomoDir := filepath.Join(rulesetDir, "mihomo")
	textDir := filepath.Join(rulesetDir, "text")
	quantumultxDir := filepath.Join(rulesetDir, "quantumultx")
	egernDir := filepath.Join(rulesetDir, "egern")
	loonDir := filepath.Join(rulesetDir, "loon")
	stashDir := filepath.Join(rulesetDir, "stash")
	shadowrocketDir := filepath.Join(rulesetDir, "shadowrocket")
	surfboardDir := filepath.Join(rulesetDir, "surfboard")
	exclaveDir := filepath.Join(rulesetDir, "exclave")
	surgeDir := filepath.Join(rulesetDir, "surge")

	fmt.Println("============================================================")
	fmt.Println("  Gins-Rules Compiler (Go)")
	fmt.Println("============================================================")

	binDir := filepath.Join(root, "bin")
	mihomoPath := findBinary("mihomo", binDir)
	if mihomoPath == "" {
		// Fallback for raw downloads without rename
		mihomoPath = findBinary("mihomo-linux-amd64", binDir)
	}
	singboxPath := findBinary("sing-box", binDir)

	fmt.Printf("\n  [Diagnostic] Root: %s\n", root)
	fmt.Printf("  [Diagnostic] BinDir: %s\n", binDir)
	fmt.Printf("  sing-box: %s (%s)\n", boolIcon(singboxPath != ""), singboxPath)
	fmt.Printf("  mihomo:   %s (%s)\n\n", boolIcon(mihomoPath != ""), mihomoPath)

	hasSingBox := singboxPath != ""
	hasMihomo := mihomoPath != ""

	outputCategories := []string{"proxy", "direct", "reject", "ip", "asn", "ai"}

	for _, dir := range []string{singboxDir, mihomoDir, textDir, quantumultxDir, egernDir, loonDir, stashDir, shadowrocketDir, surfboardDir, exclaveDir, surgeDir} {
		os.RemoveAll(dir)
		os.MkdirAll(dir, 0o755)
		for _, cat := range outputCategories {
			os.MkdirAll(filepath.Join(dir, cat), 0o755)
		}
	}

	categories := []string{"proxy", "direct", "reject", "ip", "asn"}
	stats := Stats{
		CategoryCounts: make(map[string]int),
		Formats:        12,
	}

	// Global rule collection for Xray DAT bundling
	allRules := make(map[string]map[string]Rules)
	for _, cat := range categories {
		allRules[cat] = make(map[string]Rules)
	}

	// Category-wide merged rules
	categoryMergedRules := make(map[string]Rules)
	for _, cat := range categories {
		categoryMergedRules[cat] = Rules{}
	}
	categoryMergedRules["ai"] = Rules{}

	for _, category := range categories {
		ruleNames := make(map[string]bool)

		localDir := filepath.Join(root, "source", category)
		upstreamDir := filepath.Join(root, "source", "upstream", category)

		// Fix: ASN rules are stored in the 'ip' directory but belong to the 'asn' category
		if category == "asn" {
			localDir = filepath.Join(root, "source", "ip")
			upstreamDir = filepath.Join(root, "source", "upstream", "ip")
		}

		for _, d := range []string{localDir, upstreamDir} {
			if _, err := os.Stat(d); err == nil {
				for _, f := range listTxtFiles(d) {
					ruleName := strings.TrimSuffix(filepath.Base(f), ".txt")
					// Sort ASN files into 'asn' category instead of 'ip'
					if category == "ip" && strings.HasPrefix(ruleName, "asn-") {
						continue
					}
					if category == "asn" && !strings.HasPrefix(ruleName, "asn-") {
						continue
					}
					ruleNames[ruleName] = true
				}
			}
		}

		isIP := category == "ip" || category == "asn"

		var sortedNames []string
		for name := range ruleNames {
			sortedNames = append(sortedNames, name)
		}
		sort.Strings(sortedNames)

		for _, name := range sortedNames {
			localPath := filepath.Join(localDir, name+".txt")
			if category == "asn" {
				localPath = filepath.Join(root, "source", "ip", name+".txt")
			}
			upstreamPath := filepath.Join(upstreamDir, name+".txt")
			if category == "asn" {
				upstreamPath = filepath.Join(root, "source", "upstream", "ip", name+".txt")
			}

			var rules Rules
			if _, err := os.Stat(localPath); err == nil {
				rules = parseSource(localPath)
			}
			if _, err := os.Stat(upstreamPath); err == nil {
				uRules := parseSource(upstreamPath)
				rules = mergeRules(rules, uRules)
			}

			if category != "proxy" {
				rules = sanitizeRules(rules)
			}

			total := len(rules.DomainSuffix) + len(rules.Domain) +
				len(rules.DomainKeyword) + len(rules.DomainRegex) + len(rules.IPCIDR) +
				len(rules.ProcessName) + len(rules.UserAgent)
			if total == 0 {
				continue
			}

			jsonPath := compileSingBoxJSON(name, rules, filepath.Join(singboxDir, category))
			srsOK := false
			if hasSingBox {
				srsOK = compileSingBoxSRS(jsonPath, filepath.Join(singboxDir, category), singboxPath)
			}

			mihomoMode := detectMihomoRuleMode(rules, isIP)
			yamlPath := compileMihomoYAML(name, rules, filepath.Join(mihomoDir, category), mihomoMode)
			mrsOK := false
			if hasMihomo {
				if !mihomoMode.isEmpty {
					mrsOK = compileMihomoMRS(yamlPath, filepath.Join(mihomoDir, category), mihomoMode.behavior, mihomoPath)
				}
			}

			stashYAML := compileMihomoYAML(name, rules, filepath.Join(stashDir, category), mihomoMode)
			if hasMihomo && mrsOK {
				compileMihomoMRS(stashYAML, filepath.Join(stashDir, category), mihomoMode.behavior, mihomoPath)
			}

			count := compileTextList(name, rules, filepath.Join(textDir, category), isIP)
			compileQuanXList(name, rules, filepath.Join(quantumultxDir, category), isIP, category)
			compileEgernYAML(name, rules, filepath.Join(egernDir, category))
			compileLoonList(name, rules, filepath.Join(loonDir, category), true, ".lsr")
			compileLoonList(name, rules, filepath.Join(shadowrocketDir, category), true, ".list")
			compileShadowrocketDomainset(name, rules, filepath.Join(shadowrocketDir, category))
			compileLoonList(name, rules, filepath.Join(surgeDir, category), true, ".list")
			compileSurgeDomainset(name, rules, filepath.Join(surgeDir, category))
			compileLoonList(name, rules, filepath.Join(surfboardDir, category), false, ".list")
			compileSurfboardDomainset(name, rules, filepath.Join(surfboardDir, category))
			compileExclaveRoute(name, rules, filepath.Join(exclaveDir, category))

			// Collect for Xray
			allRules[category][name] = rules

			srsIcon := "·"
			if srsOK {
				srsIcon = "✓"
				stats.SRS++
			}
			mrsIcon := "·"
			if mrsOK {
				mrsIcon = "✓"
				stats.MRS++
			}

			fmt.Printf("  [%-6s] %-20s %3d rules  srs:%s  mrs:%s\n",
				category, name, count, srsIcon, mrsIcon)

			stats.Services++
			stats.Rules += count
			if isIP {
				stats.IPRules += count
			}
			if strings.HasPrefix(name, "asn-") {
				stats.ASNFiles++
			}
			stats.CategoryCounts[category] += count

			// Add to big merge
			catRules := categoryMergedRules[category]
			catRules = mergeRules(catRules, rules)
			categoryMergedRules[category] = catRules

			if isAIRuleName(name) {
				aiRules := categoryMergedRules["ai"]
				aiRules = mergeRules(aiRules, rules)
				categoryMergedRules["ai"] = aiRules
				compileDerivedCategoryRule(name, rules, singboxDir, mihomoDir, textDir, quantumultxDir, egernDir, loonDir, stashDir, shadowrocketDir, surfboardDir, exclaveDir, surgeDir, hasSingBox, hasMihomo, singboxPath, mihomoPath)
			}
		}
	}

	// Final step: Compile merged rules for each category
	fmt.Println("\n  [Finalizing] Generating merged rule-sets...")
	for _, category := range outputCategories {
		fullRules := categoryMergedRules[category]
		if len(fullRules.DomainSuffix)+len(fullRules.Domain)+len(fullRules.DomainKeyword)+len(fullRules.DomainRegex)+len(fullRules.IPCIDR)+len(fullRules.ProcessName)+len(fullRules.UserAgent) == 0 {
			continue
		}

		name := category
		isIP := category == "ip" || category == "asn"
		mihomoMode := detectMihomoRuleMode(fullRules, isIP)

		// Compile to all formats
		jsonPath := compileSingBoxJSON(name, fullRules, filepath.Join(singboxDir, category))
		if hasSingBox {
			compileSingBoxSRS(jsonPath, filepath.Join(singboxDir, category), singboxPath)
		}
		yamlPath := compileMihomoYAML(name, fullRules, filepath.Join(mihomoDir, category), mihomoMode)
		if hasMihomo {
			if !mihomoMode.isEmpty {
				compileMihomoMRS(yamlPath, filepath.Join(mihomoDir, category), mihomoMode.behavior, mihomoPath)
			}
		}
		stashYAML := compileMihomoYAML(name, fullRules, filepath.Join(stashDir, category), mihomoMode)
		if hasMihomo {
			if !mihomoMode.isEmpty {
				compileMihomoMRS(stashYAML, filepath.Join(stashDir, category), mihomoMode.behavior, mihomoPath)
			}
		}

		compileTextList(name, fullRules, filepath.Join(textDir, category), isIP)
		compileQuanXList(name, fullRules, filepath.Join(quantumultxDir, category), isIP, category)
		compileEgernYAML(name, fullRules, filepath.Join(egernDir, category))
		compileLoonList(name, fullRules, filepath.Join(loonDir, category), true, ".lsr")
		compileLoonList(name, fullRules, filepath.Join(shadowrocketDir, category), true, ".list")
		compileShadowrocketDomainset(name, fullRules, filepath.Join(shadowrocketDir, category))
		compileLoonList(name, fullRules, filepath.Join(surgeDir, category), true, ".list")
		compileSurgeDomainset(name, fullRules, filepath.Join(surgeDir, category))
		compileLoonList(name, fullRules, filepath.Join(surfboardDir, category), false, ".list")
		compileSurfboardDomainset(name, fullRules, filepath.Join(surfboardDir, category))
		compileExclaveRoute(name, fullRules, filepath.Join(exclaveDir, category))

		fmt.Printf("  ✅ [%-6s] Created full merged rule-set: %s\n", category, name)
	}

	// Final step: Bundle everything into Xray DAT files and MMDB
	fmt.Println("\n  [Assets] Packing binary assets (DAT & MMDB)...")
	if err := compileXrayDAT(allRules, rulesetDir); err != nil {
		fmt.Printf("  [Error] Xray packing failed: %v\n", err)
	}
	if err := compileMMDB(allRules, rulesetDir); err != nil {
		fmt.Printf("  [Error] MMDB packing failed: %v\n", err)
	}

	for _, formatDir := range []string{"singbox", "mihomo", "text", "quantumultx", "egern", "loon", "stash", "shadowrocket", "surfboard", "exclave", "xray"} {
		for _, cat := range outputCategories {
			dir := filepath.Join(rulesetDir, formatDir, cat)
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
	fmt.Printf("  Done! %d files, %d total rules\n", stats.Services, stats.Rules)
	fmt.Printf("  SRS: %d/%d  MRS: %d/%d\n", stats.SRS, stats.Services, stats.MRS, stats.Services)
	fmt.Printf("  Output: %s\n", compiledDir)
	fmt.Println("============================================================")

	// Save build summary for GHA webhook
	summaryPath := filepath.Join(rulesetDir, "build-summary.json")
	summaryBytes, _ := json.MarshalIndent(stats, "", "  ")
	os.WriteFile(summaryPath, summaryBytes, 0o644)

	copyParsersJS(root, compiledDir)
}

func copyParsersJS(root string, compiledDir string) {
	parsers := []string{"QX-Resource-Parser.js", "Loon-Resource-Parser.js", "geo_location_checker.js"}
	dashboardPublic := filepath.Join(root, "dashboard", "public")
	os.MkdirAll(dashboardPublic, 0o755)

	for _, p := range parsers {
		srcPath := filepath.Join(root, "source", p)
		data, err := os.ReadFile(srcPath)
		if err == nil {
			// Copy to compiled/ for R2 Sync
			os.WriteFile(filepath.Join(compiledDir, p), data, 0o644)
			// Copy to dashboard/public/ for Astro Build & Worker Assets
			os.WriteFile(filepath.Join(dashboardPublic, p), data, 0o644)
			fmt.Printf("  [SUCCESS] Distributed %s to compiled/ and dashboard/public/\n", p)
		}
	}
}

func isAIRuleName(name string) bool {
	for _, candidate := range aiRuleNames {
		if name == candidate {
			return true
		}
	}
	return false
}

func compileDerivedCategoryRule(
	name string,
	rules Rules,
	singboxDir string,
	mihomoDir string,
	textDir string,
	quanxDir string,
	egernDir string,
	loonDir string,
	stashDir string,
	shadowrocketDir string,
	surfboardDir string,
	exclaveDir string,
	surgeDir string,
	hasSingBox bool,
	hasMihomo bool,
	singboxPath string,
	mihomoPath string,
) {
	const category = "ai"

	jsonPath := compileSingBoxJSON(name, rules, filepath.Join(singboxDir, category))
	if hasSingBox {
		compileSingBoxSRS(jsonPath, filepath.Join(singboxDir, category), singboxPath)
	}

	mihomoMode := detectMihomoRuleMode(rules, false)
	yamlPath := compileMihomoYAML(name, rules, filepath.Join(mihomoDir, category), mihomoMode)
	if hasMihomo && !mihomoMode.isEmpty {
		compileMihomoMRS(yamlPath, filepath.Join(mihomoDir, category), mihomoMode.behavior, mihomoPath)
	}

	stashYAML := compileMihomoYAML(name, rules, filepath.Join(stashDir, category), mihomoMode)
	if hasMihomo && !mihomoMode.isEmpty {
		compileMihomoMRS(stashYAML, filepath.Join(stashDir, category), mihomoMode.behavior, mihomoPath)
	}

	compileTextList(name, rules, filepath.Join(textDir, category), false)
	compileQuanXList(name, rules, filepath.Join(quanxDir, category), false, category)
	compileEgernYAML(name, rules, filepath.Join(egernDir, category))
	compileLoonList(name, rules, filepath.Join(loonDir, category), true, ".lsr")
	compileLoonList(name, rules, filepath.Join(shadowrocketDir, category), true, ".list")
	compileShadowrocketDomainset(name, rules, filepath.Join(shadowrocketDir, category))
	compileLoonList(name, rules, filepath.Join(surgeDir, category), true, ".list")
	compileSurgeDomainset(name, rules, filepath.Join(surgeDir, category))
	compileLoonList(name, rules, filepath.Join(surfboardDir, category), false, ".list")
	compileSurfboardDomainset(name, rules, filepath.Join(surfboardDir, category))
	compileExclaveRoute(name, rules, filepath.Join(exclaveDir, category))
}

func sanitizeRules(rules Rules) Rules {
	forceProxy := []string{
		"browserleaks.com", "browserleaks.org",
		"ipleak.net", "ipleak.vip",
		"ipinfo.io", "ip.sb",
		"whoer.net", "dnsleaktest.com",
		"tiktok.com", "tiktokv.com", "tiktokcdn.com",
		"byteoversea.com", "ibytedtos.com", "ipstatp.com",
		"muscdn.com", "musical.ly", "tik-tokapi.com",
	}

	isForce := func(d string) bool {
		for _, p := range forceProxy {
			if d == p || strings.HasSuffix(d, "."+p) {
				return true
			}
		}
		return false
	}

	var newDomain []string
	for _, d := range rules.Domain {
		if !isForce(d) {
			newDomain = append(newDomain, d)
		}
	}

	var newSuffix []string
	for _, d := range rules.DomainSuffix {
		if !isForce(d) {
			newSuffix = append(newSuffix, d)
		}
	}

	rules.Domain = newDomain
	rules.DomainSuffix = newSuffix
	return rules
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
		case strings.HasPrefix(line, "process:"):
			r.ProcessName = append(r.ProcessName, line[8:])
		case strings.HasPrefix(line, "user-agent:"):
			r.UserAgent = append(r.UserAgent, line[11:])
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
	a.ProcessName = append(a.ProcessName, b.ProcessName...)
	a.UserAgent = append(a.UserAgent, b.UserAgent...)

	a.DomainSuffix = unique(a.DomainSuffix)
	a.Domain = unique(a.Domain)
	a.DomainKeyword = unique(a.DomainKeyword)
	a.DomainRegex = unique(a.DomainRegex)
	a.IPCIDR = unique(a.IPCIDR)
	a.ProcessName = unique(a.ProcessName)
	a.UserAgent = unique(a.UserAgent)
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

func compileMihomoYAML(name string, rules Rules, outDir string, mode mihomoRuleMode) string {
	var lines []string
	lines = append(lines, fmt.Sprintf("# Gins-Rules: %s", name))
	lines = append(lines, "# Auto-generated, do not edit")
	lines = append(lines, "")
	lines = append(lines, "payload:")

	if mode.behavior == "ipcidr" {
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

func detectMihomoRuleMode(rules Rules, categoryIsIP bool) mihomoRuleMode {
	hasIP := len(rules.IPCIDR) > 0
	hasDomain := len(rules.DomainSuffix) > 0 || len(rules.Domain) > 0

	if categoryIsIP || (hasIP && !hasDomain) {
		return mihomoRuleMode{
			isIP:     true,
			behavior: "ipcidr",
			isEmpty:  !hasIP,
		}
	}

	return mihomoRuleMode{
		isIP:     false,
		behavior: "domain",
		isEmpty:  !hasDomain,
	}
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

func compileQuanXList(name string, rules Rules, outDir string, isIP bool, category string) {
	var lines []string

	policy := "Proxy"
	switch category {
	case "direct":
		policy = "Direct"
	case "reject":
		policy = "Reject"
	}

	for _, d := range rules.DomainSuffix {
		lines = append(lines, fmt.Sprintf("host-suffix,%s,%s", d, policy))
	}
	for _, d := range rules.Domain {
		lines = append(lines, fmt.Sprintf("host,%s,%s", d, policy))
	}
	for _, d := range rules.DomainKeyword {
		lines = append(lines, fmt.Sprintf("host-keyword,%s,%s", d, policy))
	}
	for _, cidr := range rules.IPCIDR {
		if strings.Contains(cidr, ":") {
			lines = append(lines, fmt.Sprintf("ip6-cidr,%s,%s", cidr, policy))
		} else {
			lines = append(lines, fmt.Sprintf("ip-cidr,%s,%s", cidr, policy))
		}
	}
	for _, ua := range rules.UserAgent {
		lines = append(lines, fmt.Sprintf("USER-AGENT,%s,%s", ua, policy))
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
	quoteYAMLString := func(value string) string {
		return strconv.Quote(value)
	}

	appendStringSet := func(lines []string, key string, values []string) []string {
		if len(values) == 0 {
			return lines
		}
		lines = append(lines, key+":")
		for _, value := range values {
			lines = append(lines, fmt.Sprintf("  - %s", quoteYAMLString(value)))
		}
		return lines
	}

	var lines []string
	lines = append(lines, "# Gins-Rules: "+name)
	lines = append(lines, "# Optimized Egern Rule Set")
	lines = append(lines, "")

	lines = appendStringSet(lines, "domain_suffix_set", rules.DomainSuffix)
	lines = appendStringSet(lines, "domain_set", rules.Domain)
	lines = appendStringSet(lines, "domain_keyword_set", rules.DomainKeyword)
	lines = appendStringSet(lines, "domain_regex_set", rules.DomainRegex)

	var v4, v6 []string
	for _, cidr := range rules.IPCIDR {
		if strings.Contains(cidr, ":") {
			v6 = append(v6, cidr)
		} else {
			v4 = append(v4, cidr)
		}
	}
	lines = appendStringSet(lines, "ip_cidr_set", v4)
	lines = appendStringSet(lines, "ip_cidr6_set", v6)
	lines = appendStringSet(lines, "user_agent_set", rules.UserAgent)

	content := strings.Join(lines, "\n") + "\n"
	os.WriteFile(filepath.Join(outDir, name+".yaml"), []byte(content), 0o644)
}

func compileLoonList(name string, rules Rules, outDir string, includeSpecial bool, suffix string) {
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
	if includeSpecial {
		for _, p := range rules.ProcessName {
			lines = append(lines, fmt.Sprintf("PROCESS-NAME,%s", p))
		}
		for _, ua := range rules.UserAgent {
			lines = append(lines, fmt.Sprintf("USER-AGENT,%s", ua))
		}
	}

	content := strings.Join(lines, "\n") + "\n"
	listPath := filepath.Join(outDir, name+suffix)
	os.WriteFile(listPath, []byte(content), 0o644)
}

func compileSurgeDomainset(name string, rules Rules, outDir string) {
	var lines []string

	// Suffixes: .example.com
	for _, d := range rules.DomainSuffix {
		lines = append(lines, "."+d)
	}
	// Exact: example.com
	for _, d := range rules.Domain {
		lines = append(lines, d)
	}

	if len(lines) == 0 {
		return
	}

	content := strings.Join(lines, "\n") + "\n"
	os.WriteFile(filepath.Join(outDir, name+".domainset"), []byte(content), 0o644)
}

func compileShadowrocketDomainset(name string, rules Rules, outDir string) {
	var lines []string

	// Suffixes: .example.com (Shadowrocket Domain Set follows Surge convention)
	for _, d := range rules.DomainSuffix {
		lines = append(lines, "."+d)
	}
	// Exact: example.com
	for _, d := range rules.Domain {
		lines = append(lines, d)
	}

	if len(lines) == 0 {
		return
	}

	content := strings.Join(lines, "\n") + "\n"
	os.WriteFile(filepath.Join(outDir, name+".txt"), []byte(content), 0o644)
}

func compileSurfboardDomainset(name string, rules Rules, outDir string) {
	var lines []string

	// Domain Suffix -> .example.com
	for _, d := range rules.DomainSuffix {
		lines = append(lines, "."+d)
	}
	// Domain (Exact) -> example.com
	for _, d := range rules.Domain {
		lines = append(lines, d)
	}

	if len(lines) == 0 {
		return
	}

	content := strings.Join(lines, "\n") + "\n"
	os.WriteFile(filepath.Join(outDir, name+".txt"), []byte(content), 0o644)
}

func compileExclaveRoute(name string, rules Rules, outDir string) {
	var lines []string

	for _, d := range rules.DomainSuffix {
		lines = append(lines, "domain:"+d)
	}
	for _, d := range rules.Domain {
		lines = append(lines, "full:"+d)
	}
	for _, d := range rules.DomainKeyword {
		lines = append(lines, "keyword:"+d)
	}
	for _, d := range rules.DomainRegex {
		lines = append(lines, "regexp:"+d)
	}
	for _, c := range rules.IPCIDR {
		lines = append(lines, "ip:"+c)
	}

	content := strings.Join(lines, "\n") + "\n"
	os.WriteFile(filepath.Join(outDir, name+".list"), []byte(content), 0o644)
}
