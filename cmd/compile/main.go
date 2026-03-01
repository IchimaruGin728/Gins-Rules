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

// Rules holds parsed rules from a source file.
type Rules struct {
	DomainSuffix  []string
	Domain        []string
	DomainKeyword []string
	DomainRegex   []string
	IPCIDR        []string
}

// SingBoxRuleSet is the sing-box rule-set JSON source format (v2).
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

// Stats tracks compilation statistics.
type Stats struct {
	Files int
	Rules int
	SRS   int
	MRS   int
}

func main() {
	root := findRoot()
	sourceDir := filepath.Join(root, "source")
	compiledDir := filepath.Join(root, "compiled")

	singboxDir := filepath.Join(compiledDir, "singbox")
	mihomoDir := filepath.Join(compiledDir, "mihomo")
	textDir := filepath.Join(compiledDir, "text")
	quanxDir := filepath.Join(compiledDir, "quanx")
	egernDir := filepath.Join(compiledDir, "egern")

	fmt.Println("============================================================")
	fmt.Println("  Gins-Rules Compiler (Go)")
	fmt.Println("============================================================")

	hasSingBox := hasCommand("sing-box")
	hasMihomo := hasCommand("mihomo")
	fmt.Printf("\n  sing-box: %s\n", boolIcon(hasSingBox))
	fmt.Printf("  mihomo:   %s\n\n", boolIcon(hasMihomo))

	// Clean and create output dirs
	for _, dir := range []string{singboxDir, mihomoDir, textDir, quanxDir, egernDir} {
		os.RemoveAll(dir)
		os.MkdirAll(dir, 0o755)
		for _, cat := range []string{"proxy", "direct", "reject", "ip"} {
			os.MkdirAll(filepath.Join(dir, cat), 0o755)
		}
	}

	categories := []string{"proxy", "direct", "reject", "ip"}
	stats := Stats{}

	for _, category := range categories {
		catDir := filepath.Join(sourceDir, category)
		if _, err := os.Stat(catDir); os.IsNotExist(err) {
			continue
		}

		isIP := category == "ip"
		behavior := "domain"
		if isIP {
			behavior = "ipcidr"
		}

		files := listTxtFiles(catDir)
		for _, srcPath := range files {
			name := strings.TrimSuffix(filepath.Base(srcPath), ".txt")
			rules := parseSource(srcPath)
			total := len(rules.DomainSuffix) + len(rules.Domain) +
				len(rules.DomainKeyword) + len(rules.DomainRegex) + len(rules.IPCIDR)
			if total == 0 {
				continue
			}

			// sing-box JSON + SRS
			jsonPath := compileSingBoxJSON(name, rules, filepath.Join(singboxDir, category))
			srsOK := false
			if hasSingBox {
				srsOK = compileSingBoxSRS(jsonPath, filepath.Join(singboxDir, category))
			}

			// Mihomo YAML + MRS
			yamlPath := compileMihomoYAML(name, rules, filepath.Join(mihomoDir, category), isIP)
			mrsOK := false
			if hasMihomo {
				mrsOK = compileMihomoMRS(yamlPath, filepath.Join(mihomoDir, category), behavior)
			}

			// Text list (Surge/Loon/SR)
			count := compileTextList(name, rules, filepath.Join(textDir, category), isIP)

			// QuantumultX native
			compileQuanXList(name, rules, filepath.Join(quanxDir, category), isIP)

			// Egern native YAML
			compileEgernYAML(name, rules, filepath.Join(egernDir, category), isIP)

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

	// Generate manifests
	for _, formatDir := range []string{"singbox", "mihomo", "text", "quanx", "egern"} {
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

// findRoot finds the project root by locating go.mod.
func findRoot() string {
	// Try executable directory first, then working directory
	exe, _ := os.Executable()
	dir := filepath.Dir(exe)

	// Walk up to find go.mod
	for d := dir; d != "/"; d = filepath.Dir(d) {
		if _, err := os.Stat(filepath.Join(d, "go.mod")); err == nil {
			return d
		}
	}

	// Fallback: working directory
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

func hasCommand(name string) bool {
	_, err := exec.LookPath(name)
	return err == nil
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

// parseSource parses a source rule file.
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
			r.DomainSuffix = append(r.DomainSuffix, line)
		}
	}
	return r
}

// compileSingBoxJSON generates a sing-box rule-set JSON source file.
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

// compileSingBoxSRS compiles JSON to binary .srs using sing-box CLI.
func compileSingBoxSRS(jsonPath, outDir string) bool {
	name := strings.TrimSuffix(filepath.Base(jsonPath), ".json")
	srsPath := filepath.Join(outDir, name+".srs")
	cmd := exec.Command("sing-box", "rule-set", "compile", jsonPath, "-o", srsPath)
	output, err := cmd.CombinedOutput()
	if err != nil {
		fmt.Fprintf(os.Stderr, "  [ERROR] sing-box compile %s: %v\n  Output: %s\n", name, err, string(output))
		return false
	}
	return true
}

// compileMihomoYAML generates a Mihomo-compatible YAML rule-provider file.
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
		for _, d := range rules.DomainSuffix {
			lines = append(lines, fmt.Sprintf("  - '+.%s'", d))
		}
		for _, d := range rules.Domain {
			lines = append(lines, fmt.Sprintf("  - '%s'", d))
		}
		for _, d := range rules.DomainKeyword {
			lines = append(lines, fmt.Sprintf("  - '+.%s'", d))
		}
	}

	yamlPath := filepath.Join(outDir, name+".yaml")
	os.WriteFile(yamlPath, []byte(strings.Join(lines, "\n")+"\n"), 0o644)
	return yamlPath
}

// compileMihomoMRS compiles YAML to binary .mrs using mihomo CLI.
func compileMihomoMRS(yamlPath, outDir, behavior string) bool {
	name := strings.TrimSuffix(filepath.Base(yamlPath), ".yaml")
	mrsPath := filepath.Join(outDir, name+".mrs")
	cmd := exec.Command("mihomo", "convert-ruleset", behavior, "yaml", yamlPath, mrsPath)
	output, err := cmd.CombinedOutput()
	if err != nil {
		fmt.Fprintf(os.Stderr, "  [ERROR] mihomo compile %s: %v\n  Output: %s\n", name, err, string(output))
		return false
	}
	return true
}

// compileTextList generates a text .list file for Loon/QX/Shadowrocket.
func compileTextList(name string, rules Rules, outDir string, isIP bool) int {
	var lines []string

	if isIP {
		for _, cidr := range rules.IPCIDR {
			prefix := "IP-CIDR"
			if strings.Contains(cidr, ":") {
				prefix = "IP-CIDR6"
			}
			lines = append(lines, fmt.Sprintf("%s,%s", prefix, cidr))
		}
	} else {
		for _, d := range rules.DomainSuffix {
			lines = append(lines, fmt.Sprintf("DOMAIN-SUFFIX,%s", d))
		}
		for _, d := range rules.Domain {
			lines = append(lines, fmt.Sprintf("DOMAIN,%s", d))
		}
		for _, d := range rules.DomainKeyword {
			lines = append(lines, fmt.Sprintf("DOMAIN-KEYWORD,%s", d))
		}
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

// compileQuanXList generates a QuantumultX native format .list file.
func compileQuanXList(name string, rules Rules, outDir string, isIP bool) {
	var lines []string

	if isIP {
		for _, cidr := range rules.IPCIDR {
			if strings.Contains(cidr, ":") {
				lines = append(lines, fmt.Sprintf("ip6-cidr, %s", cidr))
			} else {
				lines = append(lines, fmt.Sprintf("ip-cidr, %s", cidr))
			}
		}
	} else {
		for _, d := range rules.DomainSuffix {
			lines = append(lines, fmt.Sprintf("host-suffix, %s", d))
		}
		for _, d := range rules.Domain {
			lines = append(lines, fmt.Sprintf("host, %s", d))
		}
		for _, d := range rules.DomainKeyword {
			lines = append(lines, fmt.Sprintf("host-keyword, %s", d))
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

// compileEgernYAML generates an Egern native YAML rule set file.
func compileEgernYAML(name string, rules Rules, outDir string, isIP bool) {
	var lines []string

	if isIP {
		if len(rules.IPCIDR) > 0 {
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
					lines = append(lines, fmt.Sprintf("  - \"%s\"", c))
				}
			}
		}
	} else {
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
		if len(rules.DomainRegex) > 0 {
			lines = append(lines, "domain_regex_set:")
			for _, d := range rules.DomainRegex {
				lines = append(lines, fmt.Sprintf("  - \"%s\"", d))
			}
		}
	}

	yamlPath := filepath.Join(outDir, name+".yaml")
	os.WriteFile(yamlPath, []byte(strings.Join(lines, "\n")+"\n"), 0o644)
}
