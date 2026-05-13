package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

type RuleSet struct {
	Domain        []string `json:"domain,omitempty"`
	DomainSuffix  []string `json:"domain_suffix,omitempty"`
	DomainKeyword []string `json:"domain_keyword,omitempty"`
	DomainRegex   []string `json:"domain_regex,omitempty"`
	DomainWildcard []string `json:"domain_wildcard,omitempty"`
	IPCidr        []string `json:"ip_cidr,omitempty"`
	IPAsn         []string `json:"ip_asn,omitempty"`
	ProcessName   []string `json:"process_name,omitempty"`
	UserAgent     []string `json:"user_agent,omitempty"`
}

type Intermediate struct {
	Version    int                            `json:"version"`
	Timestamp  string                         `json:"timestamp"`
	Categories map[string]map[string]RuleSet  `json:"categories"`
}

func main() {
	input := flag.String("input", "", "Path to intermediate.json")
	output := flag.String("output", "", "Output directory for binary formats")
	mihomoBin := flag.String("mihomo", "bin/mihomo", "Path to mihomo binary")
	singboxBin := flag.String("singbox", "sing-box", "Path to sing-box binary")
	flag.Parse()

	if *input == "" || *output == "" {
		log.Fatal("--input and --output are required")
	}

	data, err := os.ReadFile(*input)
	if err != nil {
		log.Fatalf("Failed to read intermediate.json: %v", err)
	}

	var intermediate Intermediate
	if err := json.Unmarshal(data, &intermediate); err != nil {
		log.Fatalf("Failed to parse intermediate.json: %v", err)
	}

	fmt.Printf("📦 [Binary] Processing %d categories...\n", len(intermediate.Categories))

	// Generate MRS files
	mrsCount := generateAllMRS(intermediate.Categories, *output, *mihomoBin)

	// Generate SRS files
	srsCount := generateAllSRS(intermediate.Categories, *output, *singboxBin)

	// Generate DAT files
	datCount := generateAllDAT(intermediate.Categories, *output)

	// Generate MMDB files
	mmdbCount := generateAllMMDB(intermediate.Categories, *output)

	fmt.Printf("✨ [Binary] Generated %d MRS, %d SRS, %d DAT, %d MMDB files\n", mrsCount, srsCount, datCount, mmdbCount)
}

func hasDomainOnly(rs RuleSet) bool {
	return len(rs.Domain) > 0 || len(rs.DomainSuffix) > 0 &&
		len(rs.DomainKeyword) == 0 && len(rs.DomainRegex) == 0 &&
		len(rs.DomainWildcard) == 0 && len(rs.IPCidr) == 0 &&
		len(rs.IPAsn) == 0 && len(rs.ProcessName) == 0 && len(rs.UserAgent) == 0
}

func hasIPOnly(rs RuleSet) bool {
	return (len(rs.IPCidr) > 0 || len(rs.IPAsn) > 0) &&
		len(rs.Domain) == 0 && len(rs.DomainSuffix) == 0 &&
		len(rs.DomainKeyword) == 0 && len(rs.DomainRegex) == 0 &&
		len(rs.DomainWildcard) == 0 && len(rs.ProcessName) == 0 && len(rs.UserAgent) == 0
}

func sortedKeys(m map[string]RuleSet) []string {
	keys := make([]string, 0, len(m))
	for k := range m {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	return keys
}

// writeLines writes sorted lines to a file
func writeLines(path string, lines []string) error {
	sort.Strings(lines)
	content := strings.Join(lines, "\n") + "\n"
	return os.WriteFile(path, []byte(content), 0644)
}

// ensureDir creates directory if it doesn't exist
func ensureDir(path string) error {
	return os.MkdirAll(path, 0755)
}

// ruleSetIsEmpty checks if a RuleSet has no rules
func ruleSetIsEmpty(rs RuleSet) bool {
	return len(rs.Domain) == 0 && len(rs.DomainSuffix) == 0 &&
		len(rs.DomainKeyword) == 0 && len(rs.DomainRegex) == 0 &&
		len(rs.DomainWildcard) == 0 && len(rs.IPCidr) == 0 &&
		len(rs.IPAsn) == 0 && len(rs.ProcessName) == 0 && len(rs.UserAgent) == 0
}

// classicalPayload generates classical format lines for a RuleSet
func classicalPayload(rs RuleSet) []string {
	var lines []string
	for _, s := range rs.DomainSuffix {
		lines = append(lines, "DOMAIN-SUFFIX,"+s)
	}
	for _, s := range rs.Domain {
		lines = append(lines, "DOMAIN,"+s)
	}
	for _, s := range rs.DomainKeyword {
		lines = append(lines, "DOMAIN-KEYWORD,"+s)
	}
	for _, s := range rs.DomainRegex {
		lines = append(lines, "DOMAIN-REGEX,"+s)
	}
	for _, s := range rs.DomainWildcard {
		lines = append(lines, "DOMAIN-WILDCARD,"+s)
	}
	for _, s := range rs.IPCidr {
		if strings.Contains(s, ":") {
			lines = append(lines, "IP-CIDR6,"+s)
		} else {
			lines = append(lines, "IP-CIDR,"+s)
		}
	}
	for _, s := range rs.IPAsn {
		lines = append(lines, "IP-ASN,"+s)
	}
	for _, s := range rs.ProcessName {
		lines = append(lines, "PROCESS-NAME,"+s)
	}
	sort.Strings(lines)
	return lines
}

// domainPayload generates domain text lines (.suffix format)
func domainPayload(rs RuleSet) []string {
	var lines []string
	for _, s := range rs.DomainSuffix {
		lines = append(lines, "."+s)
	}
	for _, s := range rs.Domain {
		lines = append(lines, s)
	}
	sort.Strings(lines)
	return lines
}

// ipcidrPayload generates ipcidr text lines
func ipcidrPayload(rs RuleSet) []string {
	var lines []string
	for _, s := range rs.IPCidr {
		lines = append(lines, s)
	}
	sort.Strings(lines)
	return lines
}

// singboxJSON generates sing-box JSON source for a RuleSet
// Note: ASN is not supported in sing-box rule-set format, so it's skipped
func singboxJSON(rs RuleSet) map[string]interface{} {
	rule := make(map[string]interface{})

	if len(rs.DomainSuffix) > 0 {
		sort.Strings(rs.DomainSuffix)
		rule["domain_suffix"] = rs.DomainSuffix
	}
	if len(rs.Domain) > 0 {
		sort.Strings(rs.Domain)
		rule["domain"] = rs.Domain
	}
	if len(rs.DomainKeyword) > 0 {
		sort.Strings(rs.DomainKeyword)
		rule["domain_keyword"] = rs.DomainKeyword
	}
	if len(rs.DomainRegex) > 0 {
		sort.Strings(rs.DomainRegex)
		rule["domain_regex"] = rs.DomainRegex
	}
	if len(rs.IPCidr) > 0 {
		sort.Strings(rs.IPCidr)
		rule["ip_cidr"] = rs.IPCidr
	}
	if len(rs.ProcessName) > 0 {
		sort.Strings(rs.ProcessName)
		rule["process_name"] = rs.ProcessName
	}

	return rule
}

func filepathJoin(elem ...string) string {
	return filepath.Join(elem...)
}
